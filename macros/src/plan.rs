use crate::Pattern;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Expr, Ident, Path, Token, parse::Parse, punctuated::Punctuated, spanned::Spanned};

pub struct Input {
    query: Pattern<Ident, Expr>,
    _if_token: Token![if],
    dependencies: Punctuated<Pattern<Ident, Expr>, Token![,]>,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Input {
            query: input.parse()?,
            _if_token: input.parse()?,
            dependencies: input.call(Punctuated::parse_separated_nonempty)?,
        })
    }
}

impl ToTokens for Input {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match make_plan(&self.query, &self.dependencies) {
            Ok(plan) => plan.to_tokens(tokens),
            Err(error) => error.to_compile_error().to_tokens(tokens),
        }
    }
}

pub fn plan_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    syn::parse_macro_input!(input as Input)
        .to_token_stream()
        .into()
}

struct Plan {
    vars: Vec<PlanVar>,
    first: PlanFirstStep,
    steps: Vec<PlanStep>,
    last: PlanLastStep,
}

struct PlanVar {
    name: Ident,
    index: usize,
}

type PlanFirstStep = Step<()>;
type PlanStep = Step<Ident>;
type PlanLastStep = Step<Result<Expr, Ident>>;

struct Step<Right> {
    query: Path,
    left: Ident,
    right: Right,
}

impl PlanFirstStep {
    fn first(query: Path, left: Ident) -> Self {
        Step {
            query,
            left,
            right: (),
        }
    }
}

impl PlanStep {
    fn plan(query: Path, left: Ident, right: Ident) -> Self {
        Step { query, left, right }
    }
}

impl PlanLastStep {
    fn last_var(query: Path, left: Ident, right: Ident) -> Self {
        Step {
            query,
            left,
            right: Err(right),
        }
    }

    fn last_expr(query: Path, left: Ident, right: Expr) -> Self {
        Step {
            query,
            left,
            right: Ok(right),
        }
    }
}

impl ToTokens for Plan {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let vars = &self.vars;
        let num_vars = vars.len();
        let first = self.first.to_token_stream();
        let steps = self.steps.iter().map(|step| step.to_token_stream());
        let last = self.last.to_token_stream();

        tokens.extend(quote! {{
            #(#vars)*

            ::wipple_datalog::Plan {
                vars: #num_vars,
                first: #first,
                steps: &[#(#steps),*],
                last: #last,
            }
        }});
    }
}

impl ToTokens for PlanVar {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let index = &self.index;

        tokens.extend(quote! {
            let #name = ::wipple_datalog::Var::new(#index);
        });
    }
}

impl ToTokens for PlanFirstStep {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let query = &self.query;
        let left = &self.left;

        tokens.extend(quote! {
            ::wipple_datalog::Step::first::<#query>(#left)
        });
    }
}

impl ToTokens for PlanStep {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let query = &self.query;
        let left = &self.left;
        let right = &self.right;

        tokens.extend(quote! {
            ::wipple_datalog::Step::plan::<#query>(#left, #right)
        });
    }
}

impl ToTokens for PlanLastStep {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let query = &self.query;
        let left = &self.left;

        match &self.right {
            Ok(expr) => tokens.extend(quote! {
                ::wipple_datalog::Step::last_expr::<#query>(#left, #expr)
            }),
            Err(var) => tokens.extend(quote! {
                ::wipple_datalog::Step::last_var::<#query>(#left, #var)
            }),
        }
    }
}

#[derive(Default)]
struct Index(Vec<Ident>);

impl Index {
    fn get(&mut self, ident: &Ident) -> usize {
        let index = self.0.iter().enumerate().find_map({
            let i = ident.to_string();
            move |(index, s)| (i == *s.to_string()).then_some(index)
        });

        index.unwrap_or_else(|| {
            let index = self.0.len();
            self.0.push(ident.clone());
            index
        })
    }
}

fn make_plan<'a>(
    query: &'a Pattern<Ident, Expr>,
    dependencies: impl IntoIterator<Item = &'a Pattern<Ident, Expr>>,
) -> syn::Result<Plan> {
    enum VarOrExpr {
        Var(Ident),
        Expr(Expr),
    }

    let span = query.span();

    let mut var_index = Index::default();

    let mut first = None;
    let mut steps = Vec::new();

    let mut known = Vec::new();
    let get_vars = |span, left, right, known: &[_], var_index: &mut Index| {
        let left_known = known.contains(&var_index.get(left));

        let right_known = match &right {
            VarOrExpr::Var(var) => known.contains(&var_index.get(var)).then_some(var),
            VarOrExpr::Expr(_) => None,
        };

        let (known_var, unknown_var) = match (left_known, right_known) {
            (true, _) => (left.clone(), right),
            (false, Some(right)) => (right.clone(), VarOrExpr::Var(left.clone())),
            (false, None) => {
                return Err(syn::Error::new(
                    span,
                    "must reference at least one known var",
                ));
            }
        };

        Ok((known_var, unknown_var))
    };

    let expr = |e: &Expr| {
        if let Expr::Path(path) = e {
            if let Some(ident) = path.path.get_ident() {
                return VarOrExpr::Var(ident.clone());
            }
        }

        VarOrExpr::Expr(e.clone())
    };

    for dependency in dependencies.into_iter() {
        first.get_or_insert_with(|| {
            // The first variable is specifically resolved at the start
            known.push(var_index.get(&dependency.left));

            Step::first(dependency.name.clone(), dependency.left.clone())
        });

        let (known_var, unknown_var) = get_vars(
            dependency.span(),
            &dependency.left,
            expr(&dependency.right),
            &mut known,
            &mut var_index,
        )?;

        let unknown_var = match unknown_var {
            VarOrExpr::Var(var) => var,
            VarOrExpr::Expr(expr) => {
                return Err(syn::Error::new(expr.span(), "expected var"));
            }
        };

        // Link the known and unknown vars as the next step
        steps.push(Step::plan(
            dependency.name.clone(),
            known_var,
            unknown_var.clone(),
        ));

        // After resolving this step, the var will be known
        known.push(var_index.get(&unknown_var));
    }

    let Some(first) = first else {
        return Err(syn::Error::new(query.span(), "no vars in rule"));
    };

    let last = match expr(&query.right) {
        VarOrExpr::Var(var) => {
            // Register this var if it's not already
            var_index.get(&var);

            Step::last_var(query.name.clone(), query.left.clone(), var)
        }
        VarOrExpr::Expr(expr) => Step::last_expr(query.name.clone(), query.left.clone(), expr),
    };

    // Now all the vars will be known and we can use the fact

    for var in 0..var_index.0.len() {
        if !known.contains(&var) {}
    }

    let unknown = (0..var_index.0.len())
        .filter(|var| !known.contains(var))
        .collect::<Vec<_>>();

    if !unknown.is_empty() {
        return Err(syn::Error::new(
            span,
            format!(
                "rule contains unknown vars: `{}`",
                unknown
                    .iter()
                    .map(|var| var_index.0[*var].to_string())
                    .collect::<Vec<_>>()
                    .join("`, `")
            ),
        ));
    }

    let vars = var_index
        .0
        .into_iter()
        .enumerate()
        .map(|(index, name)| PlanVar { name, index })
        .collect::<Vec<_>>();

    Ok(Plan {
        vars,
        first,
        steps,
        last,
    })
}
