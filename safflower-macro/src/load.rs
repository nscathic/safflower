use std::rc::Rc;

use proc_macro2::Span;
use quote::quote;
use safflower_core::{
    generator::Generator, parser::{Locale, ParsedData, Parser, Scope}, 
};

pub struct Loader {
    span: proc_macro2::Span,
    path: String,
}
impl syn::parse::Parse for Loader {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Get a path
        let path: syn::LitStr = input.parse()?;

        Ok(Self { 
            span: path.span(),
            path: path.value(),
        })
    }
}
impl Loader {
    pub fn collect(self) -> syn::Result<LoadedData> {
        let ParsedData { locales, scope } = Parser::new(self.path)
        .and_then(Parser::parse)
        .map_err(|e| syn::Error::new(self.span, e))?;

        if locales.is_empty() {
            return Err(syn::Error::new(
                Span::call_site(), 
                "no locales have been declared (use `!locales [L1] [L2]...`)",
            ));
        }

        Ok(LoadedData {
            locales,
            scope,
        })
    }
}

pub struct LoadedData {
    locales: Vec<Rc<Locale>>,
    scope: Scope,
}
impl quote::ToTokens for LoadedData {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let code = Generator::generate(
            &self.locales, 
            &self.scope,
        );

        tokens.extend(quote! { mod localisation { #code } });
    }
}
