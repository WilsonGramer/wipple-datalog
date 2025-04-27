use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{Expr, Path, Token, parenthesized, parse::Parse, spanned::Spanned, token};

mod derive_fact;
mod fact;
mod plan;
mod query;
mod rules;

#[proc_macro_derive(Fact)]
pub fn derive_fact(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_fact::derive_fact_macro(input)
}

#[proc_macro]
pub fn fact(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    fact::fact_macro(input)
}

#[proc_macro]
pub fn plan(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    plan::plan_macro(input)
}

#[proc_macro]
pub fn rules(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    rules::rules_macro(input)
}

#[proc_macro]
pub fn query(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    query::query_macro(input)
}

struct Pattern<T> {
    name: Path,
    _paren_token: token::Paren,
    left: T,
    _comma_token: Token![,],
    right: T,
}

impl<T: Parse> Parse for Pattern<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Pattern {
            name: input.parse()?,
            _paren_token: parenthesized!(content in input),
            left: content.parse()?,
            _comma_token: content.parse()?,
            right: content.parse()?,
        })
    }
}

impl<T> Pattern<T> {
    fn span(&self) -> Span {
        self.name.span()
    }
}

enum ExprOrPlaceholder {
    Expr(Expr),
    Placeholder,
}

impl Parse for ExprOrPlaceholder {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![_]) {
            let _comma_token = input.parse::<Token![,]>()?;
            Ok(ExprOrPlaceholder::Placeholder)
        } else {
            Ok(ExprOrPlaceholder::Expr(input.parse()?))
        }
    }
}

impl ToTokens for ExprOrPlaceholder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            ExprOrPlaceholder::Expr(expr) => quote! { Some(#expr) },
            ExprOrPlaceholder::Placeholder => quote! { None },
        });
    }
}
