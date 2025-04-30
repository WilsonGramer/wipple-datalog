import { expect, test } from "bun:test";
import { fact, rule, Val, Context } from "../src/index.ts";

class List {}
class Element {}
const firstElement = fact`${List}'s first element is ${Element}`;
const nextElement = fact`after ${Element} is ${Element}`;
const relatedLists = fact`the list ${List} is related to ${Element}`;
const relatedElements = fact`the element ${Element} is related to ${Element}`;

const rules = [
    rule("relate first elements", (xs) => {
        const x = firstElement(xs);
        const ys = relatedLists(xs);
        const y = firstElement(ys);
        return relatedElements(x, y);
    }),
    rule("relate next elements", (x1) => {
        const x2 = nextElement(x1);
        const y1 = relatedElements(x1);
        const y2 = nextElement(y1);
        return relatedElements(x2, y2);
    }),
];

test("zip", () => {
    const ctx = new Context();

    const as = new Val("as");
    const a1 = new Val("a1");
    const a2 = new Val("a2");
    const a3 = new Val("a3");

    const bs = new Val("bs");
    const b1 = new Val("b1");
    const b2 = new Val("b2");
    const b3 = new Val("b3");

    ctx.add(relatedLists(as, bs));
    ctx.add(firstElement(as, a1), nextElement(a1, a2), nextElement(a2, a3));
    ctx.add(firstElement(bs, b1), nextElement(b1, b2), nextElement(b2, b3));

    ctx.run(rules);

    ctx.print(relatedElements(undefined, undefined));

    expect(ctx.contains(relatedElements(a1, b1)));
    expect(ctx.contains(relatedElements(a2, b2)));
    expect(ctx.contains(relatedElements(a3, b3)));
    expect(!ctx.contains(relatedElements(a1, b2)));
    expect(!ctx.contains(relatedElements(a2, b3)));
});
