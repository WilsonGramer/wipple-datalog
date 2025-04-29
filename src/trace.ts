import { Fact, Val } from "./dsl.ts";
import chalk from "chalk";

export interface Query {
    fact: Fact;
    left: Val | undefined;
    right: Val | undefined;
}

export class Trace implements Query {
    constructor(
        public fact: Fact,
        public left: Val,
        public right: Val,
        public origin?: {
            rule: string;
            dependencies: Trace[];
        }
    ) {}

    public toString() {
        const result = { current: "" };
        this.write(result);
        return result.current;
    }

    private write(result: { current: string }, level = 0) {
        const indent = "  ".repeat(level);

        let label = indent + this.fact.stringFor(chalk.blue(this.left), chalk.blue(this.right));
        if (level === 0) {
            label = chalk.bold.underline(label);
        }

        result.current += label;

        if (this.origin != null) {
            let label = ` (${this.origin.rule})`;
            if (level === 0) {
                label = chalk.underline(label);
            }

            result.current += chalk.dim(label);

            for (const dependency of this.origin.dependencies) {
                result.current += "\n";
                dependency.write(result, level + 1);
            }
        }
    }
}
