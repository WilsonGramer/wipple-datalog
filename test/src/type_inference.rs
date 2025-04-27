#![cfg(test)]

use wipple_datalog::{Context, Fact, Val, fact, rules};

struct Expr;

#[derive(Fact)]
struct ForParameter(Expr, Expr);

#[derive(Fact)]
struct ForOutput(Expr, Expr);

mod is_expr {
    use super::*;

    #[derive(Fact)]
    pub(super) struct FunctionParameter(Expr, Expr);

    #[derive(Fact)]
    pub(super) struct FunctionOutput(Expr, Expr);

    #[derive(Fact)]
    pub(super) struct CallFunction(Expr, Expr);

    #[derive(Fact)]
    pub(super) struct CallParameter(Expr, Expr);
}

struct Type;

#[derive(Fact)]
struct HasType(Expr, Type);

struct TypeContext;

mod is_type {
    use super::*;

    #[derive(Fact)]
    pub(super) struct Int(TypeContext, Type);
}

rules! {
    let link_parameters =
        ForParameter(a, b) if
            is_expr::CallFunction(call, function),
            is_expr::CallParameter(call, a),
            is_expr::FunctionParameter(function, b);

    let link_returns =
        ForOutput(call, output) if
            is_expr::CallFunction(call, function),
            is_expr::FunctionOutput(function, output);

    let unify_parameters =
        HasType(b, ty) if
            ForParameter(a, b),
            HasType(a, ty);

    let unify_returns =
        HasType(call, ty) if
            ForOutput(call, output),
            HasType(output, ty);
}

#[test]
fn test_type_inference() {
    let mut ctx = Context::new();

    let function = Val::<Expr>::new("function");
    let parameter = Val::<Expr>::new("parameter");
    let call = Val::<Expr>::new("call");
    let num = Val::<Expr>::new("num");
    let int = Val::<Type>::new("int");
    let type_ctx = Val::<TypeContext>::new("type_ctx");

    ctx.add(fact!(
        is_expr::FunctionParameter(function, parameter),
        "initial"
    ));

    ctx.add(fact!(
        is_expr::FunctionOutput(function, parameter),
        "initial"
    ));

    ctx.add(fact!(is_expr::CallFunction(call, function), "initial"));

    ctx.add(fact!(is_expr::CallParameter(call, num), "initial"));

    ctx.add(fact!(is_type::Int(type_ctx, int), "initial"));

    ctx.add(fact!(HasType(num, int), "initial"));

    ctx.run(rules());

    ctx.print();
}
