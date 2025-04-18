#![cfg(test)]

use wipple_datalog::{Context, Fact, fact, rules, val};

struct Num;

#[derive(Fact)]
struct LessThan(Num, Num);

#[derive(Fact)]
struct GreaterThan(Num, Num);

rules! {
    let transitive =
        LessThan(a, c) if
            LessThan(a, b),
            LessThan(b, c);

    let inverse =
        GreaterThan(b, a) if
            LessThan(a, b);
}

#[test]
fn test_less_than() {
    let mut ctx = Context::new();

    let one = val("1");
    let two = val("2");
    let three = val("3");
    let four = val("4");

    ctx.add(fact!(LessThan(one, two)));
    ctx.add(fact!(LessThan(two, three)));
    ctx.add(fact!(LessThan(three, four)));

    ctx.run(rules());

    ctx.print();
}
