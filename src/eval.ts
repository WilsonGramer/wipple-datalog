import { Fact, Rule, Val, Var } from "./dsl.ts";
import { Query, Trace } from "./trace.ts";

export class Context {
    private facts: Map<Fact, Trace[]>;

    public constructor() {
        this.facts = new Map();
    }

    public all(): IteratorObject<Trace> {
        return this.facts.values().flatMap((traces) => traces);
    }

    public get(query: Query): Trace[] {
        if (!this.facts.has(query.fact)) {
            return [];
        }

        return this.facts
            .get(query.fact)!
            .filter(
                (trace) =>
                    (query.left == null || trace.left === query.left) &&
                    (query.right == null || trace.right === query.right)
            );
    }

    public contains(query: Trace): boolean {
        return this.get(query).length > 0;
    }

    public add(...traces: Trace[]): boolean {
        let didAdd = false;
        for (const trace of traces) {
            // Don't add duplicate facts
            if (this.contains(trace)) {
                continue;
            }

            if (!this.facts.has(trace.fact)) {
                this.facts.set(trace.fact, []);
            }

            this.facts.get(trace.fact)!.push(trace);
            didAdd = true;
        }

        return didAdd;
    }

    public run(rules: Rule[]) {
        while (true) {
            const newFacts: Trace[] = [];
            for (const rule of rules) {
                // Create a plan for the rule: "given an existing var, produce a
                // fact with it and the next var". If the rule is a tree, this
                // does a topological sort. As a result, we always have one
                // known var and one unknown var at any given time.

                const plan: [Fact, Var, Var][][] = [[[rule.fact, rule.left, rule.right]]];

                function addStep(fact: Fact, left: Var, right: Var, result: [Fact, Var, Var][]) {
                    for (const [existingFact, existingLeft, existingRight] of plan.flatMap(
                        (step) => step
                    )) {
                        if (
                            existingFact === fact &&
                            existingLeft === left &&
                            existingRight === right
                        ) {
                            return;
                        }
                    }

                    result.push([fact, left, right]);
                }

                while (true) {
                    const step: [Fact, Var, Var][] = [];
                    for (const [_, nextLeft, nextRight] of plan[plan.length - 1]) {
                        if (nextLeft.input) {
                            const [inputFact, inputVar] = nextLeft.input;
                            addStep(inputFact, inputVar, nextLeft, step);
                        }

                        if (nextRight.input) {
                            const [inputFact, inputVar] = nextRight.input;
                            addStep(inputFact, inputVar, nextRight, step);
                        }
                    }

                    if (step.length === 0) break;

                    plan.push(step);
                }

                const steps = plan.flatMap((step) => step);
                steps.reverse();

                // Now, execute the plan, storing the intermediate values in `vals`.

                const run = (
                    stepIndex: number,
                    vals: Map<Var, Val>,
                    dependencies: Trace[] = []
                ) => {
                    if (stepIndex + 1 >= steps.length) {
                        // If we've reached the end, produce an actual fact trace.

                        const trace = new Trace(
                            rule.fact,
                            vals.get(rule.left)!,
                            vals.get(rule.right)!,
                            { rule: rule.name, dependencies }
                        );

                        newFacts.push(trace);

                        return;
                    }

                    const [fact, leftVar, rightVar] = steps[stepIndex];

                    // `vals.get(...)` can return `undefined`, so if a var
                    // doesn't yet have a value, this will query all possible
                    // values.
                    const query: Query = {
                        fact,
                        left: vals.get(leftVar),
                        right: vals.get(rightVar),
                    };

                    // Try every possibility recursively (naïve evaluation)
                    for (const trace of this.get(query)) {
                        const newVals = new Map(vals);
                        newVals.set(rightVar, trace.right);

                        const newDependencies = [...dependencies, trace];

                        run(stepIndex + 1, newVals, newDependencies);
                    }
                };

                // Start with the first step

                const [fact, left, _] = steps[0];

                const query: Query = {
                    fact,
                    left: undefined,
                    right: undefined,
                };

                for (const factTrace of this.get(query)) {
                    const vals = new Map<Var, Val>();
                    vals.set(left, factTrace.left);
                    run(0, vals);
                }
            }

            const didAdd = this.add(...newFacts);
            if (!didAdd) break;

            // Keep going as long as we make progress
        }
    }

    public print() {
        for (const trace of this.all()) {
            console.log(trace.toString());
            console.log();
        }
    }
}
