#![cfg(test)]

use wipple_datalog::{Context, Fact, Val, fact, rules};

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

    let one = Val::new("1");
    let two = Val::new("2");
    let three = Val::new("3");
    let four = Val::new("4");

    ctx.add(fact!(LessThan(one, two)));
    ctx.add(fact!(LessThan(two, three)));
    ctx.add(fact!(LessThan(three, four)));

    ctx.run(rules());

    ctx.print();
}
