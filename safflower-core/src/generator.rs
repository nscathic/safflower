use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};

use crate::{ENV_LOCALE_NAME, error::ParseError, parser::{Key, Head}, shorten, validate_char};

#[cfg(test)]
mod tests;

pub struct Generator {
    locales: Vec<(syn::Ident, String)>,
    keys: Vec<Key>,
}

impl Generator {
    #[must_use] 
    /// Sets itself up.
    pub fn new(head: Head, keys: Vec<Key>) -> Self {
        let locales = head.locales()
        .into_iter()
        // Map e.g. "en_us" to "EnUs"
        // There is some trust in myself here that locales only ever contain
        // lowercase letters and '_'.
        .map(|l| {
            let id = l
            .split('_')
            .filter_map(|p| {
                // Capitalise first letter
                let mut cs = p.chars();
                cs.next().map(|c| 
                    String::from(c.to_ascii_uppercase()) 
                    + &cs.collect::<String>()
                )
            })
            .collect::<String>();

            (syn::Ident::new(&id, Span::call_site()), l.to_ascii_uppercase())
        })
        .collect::<Vec<_>>();

        Self { 
            locales, 
            keys,
        }
    }

    /// Sets the locale to the deafult one.
    /// # Safety
    /// See [`std::env::set_var`].
    pub unsafe fn set_default_locale(&self) {
        unsafe {
            std::env::set_var(ENV_LOCALE_NAME, &self.locales[0].1);
        }
    }

    /// Generates code.
    /// 
    /// # Errors
    /// If there are no defined locales.
    pub fn generate(mut self) -> Result<TokenStream, ParseError> {
        let locales = self.generate_enum();
        let getter = self.generate_getter()?;
        let setter = self.generate_setter()?;
        let keys = std::mem::take(&mut self.keys)
        .into_iter()
        .map(|key| self.generate_from_key(key))
        .collect::<Result<Vec<_>, ParseError>>()?;

        let code = quote! {
            #locales
            #getter
            #setter
            #(#keys)*
        }.into_token_stream();

        Ok(code)
    }
    
    /// Generates an enum of locales.
    fn generate_enum(&self) -> TokenStream {
        let locales = self.locales.iter().map(|(i, _)| i).collect::<Vec<_>>();
        let count = self.locales.len();

        quote! {
            #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
            pub enum Locale {
                #(#locales,)*
            }

            pub const LOCALES: [Locale; #count] = [
                #(Locale::#locales,)*
            ];
        }.into_token_stream()
    }

    /// Generates a function to get the current locale.
    fn generate_getter(&self) -> Result<TokenStream, ParseError> {
        if self.locales.is_empty() {return Err(ParseError::NoLocales); }

        let default_locale = &self.locales[0].0;
        let match_entries = self.locales
            .iter()
            .skip(1)
            .map(|(l, i)| {
                quote! {
                    #i => Locale::#l
                }
            });

        let code = quote! {
            pub fn get_locale() -> Locale {
                let Ok(var) = std::env::var(#ENV_LOCALE_NAME) else {
                    return Locale::#default_locale;
                };

                match var.as_str() {
                    #(#match_entries,)*
                    _ => Locale::#default_locale,
                }
            }
        }.into_token_stream();

        Ok(code)
    }

    /// Generates a function to set the current locale.
    fn generate_setter(&self) -> Result<TokenStream, ParseError> {
        if self.locales.is_empty() {return Err(ParseError::NoLocales); }

        let match_entries = self.locales
            .iter()
            .map(|(l, i)| quote! { Locale::#l => #i });

        let code = quote! {
            pub unsafe fn set_locale(locale: Locale) {
                let value = match locale {
                    #(#match_entries,)*
                };
                std::env::set_var(#ENV_LOCALE_NAME, value);
            }
        }.into_token_stream();

        Ok(code)
    }

    fn generate_from_key(&self, key: Key) -> Result<TokenStream, ParseError> {
        let Key { id, comment, entries } = key;

        let args = get_arguments(&entries[0])?;
        
        for (i, e) in entries.iter().enumerate().skip(1) {
            let a = get_arguments(e)?;
            assert!(
                a == args,
                "in key \"{id}\", entry {} has arguments {a:?}, but entry \
                {} has arguments {args:?}",
                self.locales[0].1,
                self.locales[i].1,
            );
        }
        let args = args.into_iter()
        .map(|a| syn::Ident::new(&a, Span::call_site()))
        .collect::<Vec<_>>();

        let id = syn::Ident::new(&id, Span::call_site());
        
        let comment = comment.map(|c| quote! {#[doc = #c]});

        let entries = entries
        .into_iter()
        .enumerate()
        .map(|(i, entry)| {
            let locale = &self.locales[i].0;
            quote! {
                Locale::#locale => format!(#entry, #(#args,)*)
            }
        });

        let code = quote! {
            #comment
            pub fn #id(
                locale: Locale,
                #(#args:impl std::fmt::Display,)*
            ) -> String {
                match locale {
                    #(#entries,)*
                }
            }
        };

        Ok(code)
    }
}

fn get_arguments(key: &str) -> Result<Vec<String>, ParseError> {
    let mut arguments = Vec::new();
    let mut argument = String::new();
    let mut opened = false;
    let mut unnamed_indexer = 0;
    let mut formatting = false;

    for c in key.chars() {
        match c {
            '{' if opened => return Err(ParseError::NestedBrace),
            '{' => { opened = true; },

            '}' if !opened => return Err(ParseError::ExtraClosingBrace),
            '}' => {
                if argument.is_empty() {
                    argument = format!("arg{unnamed_indexer}");
                    unnamed_indexer += 1;
                }
                else if argument.starts_with(|c: char| c.is_ascii_digit()) {
                    argument = format!("arg{argument}");
                }

                if !arguments.contains(&argument) {                        
                    arguments.push(argument);
                }

                argument = String::new();
                opened = false;
                formatting = false;
            }

            ':' if opened => formatting = true,

            // Don't copy the formatting part
            c if opened && !formatting => argument.push(
                validate_char(c)
                .map_err(|c| ParseError::ArgBadChar(
                    shorten(key), 
                    c,
                ))?
            ),
            
            _ => (),
        }
    }

    Ok(arguments) 
}
