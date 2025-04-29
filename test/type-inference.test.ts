import { expect, test } from "bun:test";
import { fact, rule, Val, Context } from "../src/index.ts";

class Expr {}
const functionInCall = fact`${Expr} contains function ${Expr}`;
const parameterInCall = fact`${Expr} contains parameter ${Expr}`;
const parameterInFunction = fact`${Expr} declares parameter ${Expr}`;
const outputOfFunction = fact`${Expr} declares output ${Expr}`;

class Type {}
const hasType = fact`${Expr} :: ${Type}`;

const rules = [
    rule("type of function parameter <- type of input", (call) => {
        const parameter = parameterInFunction(functionInCall(call));
        const parameterType = hasType(parameterInCall(call));
        return hasType(parameter, parameterType);
    }),
    rule("type of function call <- type of function's output", (call) => {
        const outputType = hasType(outputOfFunction(functionInCall(call)));
        return hasType(call, outputType);
    }),
];

test("type inference", () => {
    const ctx = new Context();

    const f = new Val("f : x -> x");
    const x = new Val("x");
    const call = new Val("f num");
    const num = new Val("num");
    const int = new Val("Int");

    // f : x -> x
    ctx.add(parameterInFunction(f, x));
    ctx.add(outputOfFunction(f, x));

    // f (num :: Int)
    ctx.add(functionInCall(call, f));
    ctx.add(parameterInCall(call, num));
    ctx.add(hasType(num, int));

    ctx.run(rules);

    ctx.print();

    expect(ctx.contains(hasType(x, int)));
    expect(ctx.contains(hasType(call, int)));
});
