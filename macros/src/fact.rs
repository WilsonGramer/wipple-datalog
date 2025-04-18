use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Expr, Path, Token, parenthesized, parse::Parse, token};

struct Input {
    name: Path,
    _paren_token: token::Paren,
    left: Expr,
    _comma_token: Token![,],
    right: Expr,
    description: Option<(Token![,], Expr)>,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Input {
            name: input.parse()?,
            _paren_token: parenthesized!(content in input),
            left: content.parse()?,
            _comma_token: content.parse()?,
            right: content.parse()?,
            description: (!input.is_empty())
                .then(|| syn::Result::Ok((input.parse()?, input.parse()?)))
                .transpose()?,
        })
    }
}

impl ToTokens for Input {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let left = &self.left;
        let right = &self.right;

        let description = self.description.as_ref().map_or_else(
            || quote! { None::<&str> },
            |(_, description)| quote! { Some(#description) },
        );

        tokens.extend(quote! {
            ::wipple_datalog::Fact::new::<#name>(&#left, &#right, #description)
        });
    }
}

pub fn fact_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    syn::parse_macro_input!(input as Input)
        .to_token_stream()
        .into()
}
