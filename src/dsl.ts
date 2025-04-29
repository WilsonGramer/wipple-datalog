import { Query, Trace } from "./trace.ts";

export class Var {
    public type?: Constructor;
    public stack?: string;

    constructor(public input: [Fact, Var] | undefined) {
        if (input && "captureStackTrace" in Error) {
            const error = new Error();
            Error.captureStackTrace(error, input[0]);
            this.stack = error.stack?.split("\n")[1]?.trim();
        }
    }
}

export class Val {
    constructor(public name: string) {}

    public toString() {
        return this.name;
    }
}

export interface Fact {
    (left: Var, right?: undefined): Var;
    (left: Var, right: Var): Rule;
    (left: Val, right: Val): Trace;
    (left: Val | undefined, right: Val | undefined): Query;
    stringFor(left: unknown, right: unknown): string;
}

type Constructor = new (...args: any[]) => unknown;

export const fact = (
    template: TemplateStringsArray,
    leftType: Constructor,
    rightType: Constructor
) =>
    newFact(
        leftType,
        rightType,
        (left, right) => template[0] + left + template[1] + right + template[2]
    );

export const newFact = (
    leftType: Constructor,
    rightType: Constructor,
    stringFor: Fact["stringFor"]
): Fact => {
    const debugVal = new Val("<...>");
    const debugString = stringFor(debugVal, debugVal);

    const fact = ((left, right) => {
        const checkType = (input: Var, type: Constructor) => {
            if (input.type != null && input.type !== type) {
                let message =
                    `evaluating ${debugString}: ` +
                    `expected ${input.type.name} var, but found ${type.name} var`;

                if (input.stack) {
                    message += `\nvar created ${input.stack}`;
                }

                throw new Error(message);
            }

            input.type = type;
        };

        if (left !== undefined && right !== undefined) {
            if (left instanceof Var && right instanceof Var) {
                checkType(left, leftType);
                checkType(right, rightType);
                return new Rule(fact, left, right);
            } else if (left instanceof Val && right instanceof Val) {
                return new Trace(fact, left, right);
            } else {
                throw new Error("unreachable");
            }
        } else if (left !== undefined) {
            if (left instanceof Var) {
                checkType(left, leftType);
                return new Var([fact, left]);
            } else if (left instanceof Val) {
                return {
                    fact,
                    left,
                    right: undefined,
                } satisfies Query;
            } else {
                left satisfies never;
                throw new Error("unreachable");
            }
        } else if (right !== undefined) {
            throw new Error("unreachable");
        } else {
            return {
                fact,
                left: undefined,
                right: undefined,
            } satisfies Query;
        }
    }) as Fact;

    fact.stringFor = stringFor;

    return fact;
};

export class Rule {
    public name: string = "<unknown>";

    public constructor(public fact: Fact, public left: Var, public right: Var) {}
}

export const rule = (name: string, build: (input: Var) => Rule) => {
    const rule = build(new Var(undefined));
    rule.name = name;
    return rule;
};
