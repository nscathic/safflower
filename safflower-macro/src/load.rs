use std::{fs::File, io::Read};

use quote::quote;
use safflower_core::{
    generator::Generator, 
    name::Name, 
    parser::{Key, ParsedData, Parser}, 
    reader::CharReader,
};

pub struct Loader {
    span: proc_macro2::Span,
    source: String,
}
impl syn::parse::Parse for Loader {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Get a path
        let path: syn::LitStr = input.parse()?;

        // Get the contents
        let mut source = String::new();
        File::open(path.value())
        .map_err(|e| syn::Error::new(path.span(), e))?
        .read_to_string(&mut source)
        .map_err(|e| syn::Error::new(path.span(), e.to_string()))?;

        Ok(Self { 
            span: path.span(),
            source,
        })
    }
}
impl Loader {
    pub fn collect(self) -> syn::Result<LoadedData> {
        let reader = CharReader::new(&self.source);
        let parsed = Parser::new(reader)
        .parse()
        .map_err(|e| syn::Error::new(
            self.span, 
            format!("error while parsing: {e}"), 
        ))?;

        let ParsedData { locales, keys } = parsed;

        Ok(LoadedData {
            locales,
            keys,
        })
    }
}

pub struct LoadedData {
    locales: Vec<Name>,
    keys: Vec<Key>,
}
impl quote::ToTokens for LoadedData {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let generator = Generator::new(
            self.locales.clone(), 
            self.keys.clone(),
        );

        let code = generator.generate();

        tokens.extend(quote! { mod localisation { #code } });
    }
}
