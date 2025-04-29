# Wipple Datalog Engine

A simple Datalog engine and DSL in TypeScript, intended for an experimental new [Wipple](https://github.com/wipplelang/wipple) compiler.

## Overview

This library lets you implement a [deductive database](https://en.wikipedia.org/wiki/Deductive_database) in TypeScript. A deductive database is a system where _facts_ are built up over time using _rules_.

For example, if we wanted to describe the properties of the `<` (less than) and `>` (greater than) operators, we could write these rules:

```ts
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
```

The `rules` variable defines the rules of the system:

-   `transitive`: "If `a < b` and `b < c`, then `a < c`."
-   `inverse`: "If `a < b`, then `b > a`."

Importantly, this library also exposes a _trace_ of the rules used to generate each new fact. My goal is to develop a compiler that exposes these traces to the programmer so they can better understand why errors occur.

We can evaluate our rules on a few sample numbers:

```ts
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
```

In the output, we can see facts like `4 > 1`, along with a trace:

```
4 > 1 (inverse)
  1 < 4 (transitive)
    1 < 2
    2 < 4 (transitive)
      2 < 3
      3 < 4
```

## How it works

The DSL is fully type-safe — you can't use a single variable as input to two incompatible queries. The `Num` type in the example above is never actually instantiated, it's just a marker for type inference. All variables used within a rule are typechecked when the rule is created.

Under the hood, `ctx.run(rules)` produces a plan for each rule, where each step in the plan contains one known variable and one unknown variable. Then, the plan is evaluated one variable at a time, and the resulting fact is added to the context.

The library currently uses [naïve evaluation](https://en.wikipedia.org/wiki/Datalog#Na%C3%AFve_evaluation), which means it keeps evaluating every rule on every fact until no more facts are generated. If performance becomes a concern, I will look into better evaluation methods!

## Tests

You can run the tests using [`bun test`](https://bun.sh).

## Resources

-   [Datalog in Javascript by Stepan Parunashvili](https://www.instantdb.com/essays/datalogjs)
-   [Codebase as Database: Turning the IDE Inside Out with Datalog by Pete Vilter](https://petevilter.me/post/datalog-typechecking)
