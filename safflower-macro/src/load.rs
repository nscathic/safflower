use quote::quote;
use safflower_core::{
    generator::Generator, 
    name::Name, 
    parser::{ParsedData, Parser, Scope}, 
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
        let parsed = Parser::new(self.path).map(Parser::parse);

        let ParsedData { locales, scope } = match parsed {
            Ok(Ok(pd)) => pd,
            Ok(Err(e)) | Err(e) => return Err(syn::Error::new(
                self.span, 
                e, 
            )),
        };

        Ok(LoadedData {
            locales,
            scope,
        })
    }
}

pub struct LoadedData {
    locales: Vec<Name>,
    scope: Scope,
}
impl quote::ToTokens for LoadedData {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let generator = Generator::new(
            self.locales.clone(), 
            self.scope.clone(),
        );

        let code = generator.generate();

        tokens.extend(quote! { mod localisation { #code } });
    }
}
