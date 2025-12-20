#![doc = include_str!("../readme.md")]

use std::{fs::File, io::Read};

use proc_macro::TokenStream;
use quote::quote;
use safflower_core::{generator::Generator, parser::Parser, reader::CharReader};
use syn::{parse, parse_macro_input};

use crate::text::Texter;

mod text;

#[proc_macro]
/// Loads a file from the specified path and parses it as a collection of text
/// entries (see crate documentation for details).
/// 
/// Generates an enum for Locales, and a bunch of functions to get the texts.
pub fn load(input: TokenStream) -> TokenStream {
    let (path, source) = match open_file(input) {
        Ok(r) => r,
        Err(err) => return err,
    };

    let reader = CharReader::new(&source);
    let mut parser = Parser::default();
    
    if let Err(e) = parser.parse(reader) {
        return syn::Error::new(
            path.span(), 
            format!("error in parsing {}: {}", path.value(), e), 
        )
        .into_compile_error()
        .into();
    }

    let (head, keys) = parser.extract();
    let generator = Generator::new(head, keys);
    unsafe {
        generator.set_default_locale();
    }

    let code = match generator.generate() {
        Ok(c) => c,
        Err(e) => return syn::Error::new(
            path.span(), 
            format!("error in generating code for {}: {e}", path.value()), 
        )
        .into_compile_error()
        .into()
    };

    quote! { mod localisation { #code } }.into()
}

#[proc_macro]
/// Acts similarly to `format!`, but takes a key from your previously `load!`ed
/// file instead of a string literal.
pub fn text(input: TokenStream) -> TokenStream {
    let code = parse_macro_input!(input as Texter);
    quote! { #code }.into()
}


fn open_file(
    input: TokenStream,
) -> Result<(syn::LitStr, String), TokenStream> {
    // Get the path
    let path: syn::LitStr = parse(input)
    .map_err(
        |err| syn::Error::new(
            err.span(), 
            "load macro takes a file path, as a string literal"
        )
        .into_compile_error()
    )?;

    // Get the file
    let mut file = File::open(path.value())
    .map_err(
        |err| syn::Error::new(
            path.span(), 
            format!("error in opening {}: {}", path.value(), err), 
        )
        .into_compile_error()
    )?;

    let mut buf = String::new(); 
    file.read_to_string(&mut buf)
    .map_err(
        |err| syn::Error::new(
            path.span(), 
            format!("error in reading {}: {}", path.value(), err), 
        )
        .into_compile_error()
    )?;

    Ok((path, buf))
}
