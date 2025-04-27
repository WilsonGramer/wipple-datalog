use crate::Pattern;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Ident, Path, Token, parse::Parse, punctuated::Punctuated};

pub struct Input {
    query: Pattern<Ident>,
    _if_token: Token![if],
    dependencies: Punctuated<Pattern<Ident>, Token![,]>,
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
    last: PlanStep,
}

struct PlanVar {
    name: Ident,
    index: usize,
}

type PlanFirstStep = Step<()>;
type PlanStep = Step<Ident>;

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
    fn new(query: Path, left: Ident, right: Ident) -> Self {
        Step { query, left, right }
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
            ::wipple_datalog::Step::new::<#query>(#left, #right)
        });
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
    query: &'a Pattern<Ident>,
    dependencies: impl IntoIterator<Item = &'a Pattern<Ident>>,
) -> syn::Result<Plan> {
    let span = query.span();

    let mut var_index = Index::default();

    let mut first = None;
    let mut steps = Vec::new();

    let mut known = Vec::new();
    let get_vars = |span, left, right, known: &[_], var_index: &mut Index| {
        let left_known = known.contains(&var_index.get(left));

        let right_known = known.contains(&var_index.get(right)).then_some(right);

        let (known_var, unknown_var) = match (left_known, right_known) {
            (true, _) => (left, right),
            (false, Some(right)) => (right, left),
            (false, None) => {
                return Err(syn::Error::new(
                    span,
                    "must reference at least one known var",
                ));
            }
        };

        Ok((known_var, unknown_var))
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
            &dependency.right,
            &mut known,
            &mut var_index,
        )?;

        // Link the known and unknown vars as the next step
        steps.push(Step::new(
            dependency.name.clone(),
            known_var.clone(),
            unknown_var.clone(),
        ));

        // After resolving this step, the var will be known
        known.push(var_index.get(unknown_var));
    }

    let Some(first) = first else {
        return Err(syn::Error::new(query.span(), "no vars in rule"));
    };

    // Register the last var if it's not already
    var_index.get(&query.right);

    let last = Step::new(query.name.clone(), query.left.clone(), query.right.clone());

    // Now all the vars will be known and we can use the fact
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
