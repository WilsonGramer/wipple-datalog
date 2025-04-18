use crate::{ExprOrPlaceholder, Pattern};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::Parse;

struct Input(Pattern<ExprOrPlaceholder, ExprOrPlaceholder>);

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Input(input.parse()?))
    }
}

impl ToTokens for Input {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.0.name;
        let left = &self.0.left;
        let right = &self.0.right;

        tokens.extend(quote! {{
            ::wipple_datalog::Query::new::<#name>(#left, #right)
        }});
    }
}

pub fn query_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    syn::parse_macro_input!(input as Input)
        .to_token_stream()
        .into()
}
