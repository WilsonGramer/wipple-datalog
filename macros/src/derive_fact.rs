use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Path, Token, Type, Visibility, parenthesized, parse::Parse, token};

struct Input {
    _vis: Visibility,
    _struct_token: Token![struct],
    name: Path,
    _paren_token: token::Paren,
    left: Type,
    _comma_token: Token![,],
    right: Type,
    _semi_token: Token![;],
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Input {
            _vis: input.parse()?,
            _struct_token: input.parse()?,
            name: input.parse()?,
            _paren_token: parenthesized!(content in input),
            left: content.parse()?,
            _comma_token: content.parse()?,
            right: content.parse()?,
            _semi_token: input.parse()?,
        })
    }
}

impl ToTokens for Input {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let left = &self.left;
        let right = &self.right;

        tokens.extend(quote! {
            impl ::wipple_datalog::BuildQuery for #name {
                type Left = #left;
                type Right = #right;

                const NAME: &str = stringify!(#name);
            }
        });
    }
}

pub fn derive_fact_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    syn::parse_macro_input!(input as Input)
        .to_token_stream()
        .into()
}
