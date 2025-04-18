use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Attribute, Ident, Token, Visibility, parse::Parse};

struct Input {
    items: Vec<RuleItem>,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse()?);
        }

        Ok(Input { items })
    }
}

impl ToTokens for Input {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let items = self.items.iter();

        let labels = self.items.iter().map(|item| &item.label);

        tokens.extend(quote! {
            #(#items)*

            pub fn rules() -> ::wipple_datalog::Rules {
                ::wipple_datalog::Rules::new()
                    #(.add::<#labels>())*
            }
        });
    }
}

struct RuleItem {
    attrs: Vec<Attribute>,
    vis: Visibility,
    _let_token: Token![let],
    label: Ident,
    _eq_token: Token![=],
    rule: crate::plan::Input,
    _semi_token: Token![;],
}

impl Parse for RuleItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(RuleItem {
            attrs: input.call(Attribute::parse_outer)?,
            vis: input.parse()?,
            _let_token: input.parse()?,
            label: input.parse()?,
            _eq_token: input.parse()?,
            rule: input.parse()?,
            _semi_token: input.parse()?,
        })
    }
}

impl ToTokens for RuleItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attrs = &self.attrs;
        let vis = &self.vis;
        let label = &self.label;
        let rule = &self.rule;

        tokens.extend(quote! {
            #(#attrs)*
            #[allow(non_camel_case_types)]
            #vis struct #label;

            impl ::wipple_datalog::BuildRule for #label {
                const LABEL: &str = stringify!(#label);
                const PLAN: ::wipple_datalog::Plan = #rule;
            }
        });
    }
}

pub fn rules_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    syn::parse_macro_input!(input as Input)
        .to_token_stream()
        .into()
}
