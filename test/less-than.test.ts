import { expect, test } from "bun:test";
import { fact, rule, Val, Context } from "../src/index.ts";

class Num {}
const lessThan = fact`${Num} < ${Num}`;
const greaterThan = fact`${Num} > ${Num}`;

const rules = [
    rule("transitive", (a) => {
        const b = lessThan(a);
        const c = lessThan(b);
        return lessThan(a, c);
    }),
    rule("inverse", (a) => {
        const b = lessThan(a);
        return greaterThan(b, a);
    }),
];

test("less than", () => {
    const ctx = new Context();

    const one = new Val("1");
    const two = new Val("2");
    const three = new Val("3");
    const four = new Val("4");

    ctx.add(lessThan(one, two));
    ctx.add(lessThan(two, three));
    ctx.add(lessThan(three, four));

    ctx.run(rules);

    ctx.print();

    expect(ctx.contains(greaterThan(four, one)));
});
