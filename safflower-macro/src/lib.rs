#![doc = include_str!("../readme.md")]

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod load;
mod text;

use load::Loader;
use text::Texter;

#[proc_macro]
/// Loads a file from the specified path and parses it as a collection of text
/// entries (see crate documentation for more details).
/// 
/// Generates an enum for locales, and a bunch of functions to get the texts.
/// 
/// ## File
/// The file must declare locales up top, with `!locales` followed by 
/// whitespace-separated names, and then list entries as `KEY: LOC "VALUE"`. 
/// Whitespace is completely ignored
/// 
/// ## Locales
/// The locales come from the config, so e.g. `!locales en es fr` would give 
/// you three locales, and an enum with the variants `En`, `Es`, and `Fr`, in 
/// that order.
pub fn load(input: TokenStream) -> TokenStream {
    let loader = parse_macro_input!(input as Loader);
    let data = match loader.collect() {
        Ok(l) => l,
        Err(e) => return e.into_compile_error().into(),
    };

    quote! { #data }.into()
}

#[proc_macro]
/// Acts similarly to `format!`, but takes a key from your previously `load!`ed
/// file instead of a string literal.
pub fn text(input: TokenStream) -> TokenStream {
    let code = parse_macro_input!(input as Texter);
    quote! { #code }.into()
}
