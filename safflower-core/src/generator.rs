use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};

use crate::{LOCALE_FAILURE_MESSAGE, error::ParseError, parser::{Key, Head}, shorten, validate_char};

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

    /// Generates code.
    /// 
    /// # Errors
    /// If there are no defined locales.
    pub fn generate(mut self) -> Result<TokenStream, ParseError> {
        let locales = self.generate_enum()?;
        let getter = Self::generate_getter();
        let setter = Self::generate_setter();
        
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
    
    /// Generates an enum of locales, and a static var to keep it.
    fn generate_enum(&self) -> Result<TokenStream, ParseError> {
        if self.locales.is_empty() {return Err(ParseError::NoLocales); }

        let locales = self.locales.iter().map(|(i, _)| i).collect::<Vec<_>>();
        let default = locales[0];
        let count = self.locales.len();

        // quote! {
        //     /// The locales available.
        //     #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        //     pub enum Locale {
        //         #(#locales,)*
        //     }

        //     /// All locales, in the order they were declared.
        //     pub const LOCALES: [Locale; #count] = [
        //         #(Locale::#locales,)*
        //     ];

        //     /// The current locale.
        //     pub static LOCALE: std::sync::Mutex<Locale> = 
        //         std::sync::Mutex::new(Locale::#default);
        // }.into_token_stream()

        let code = quote! {
            #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
            pub enum Locale {
                #(#locales,)*
            }

            pub const LOCALES: [Locale; #count] = [
                #(Locale::#locales,)*
            ];

            pub static LOCALE: std::sync::Mutex<Locale> = 
                std::sync::Mutex::new(Locale::#default);
        }.into_token_stream();

        Ok(code)
    }

    /// Generates a function to get the current locale.
    fn generate_getter() -> TokenStream {
        /*
            /// Gets the current locale. As this calls `Mutex::lock()`, it will
            /// block the thread until it is safe to access. 
            /// 
            /// # Panic 
            /// It will panic if the `Mutex` has been poisoned. See 
            /// [`std::sync::Mutex`].
        */          
        quote! {
            pub fn get_locale() -> Locale {
                *LOCALE
                .lock()
                .expect(#LOCALE_FAILURE_MESSAGE)
            }
        }
    }

    /// Generates a function to set the current locale.
    fn generate_setter() -> TokenStream {
        quote! {
            pub fn set_locale(locale: Locale) {
                *LOCALE
                .lock()
                .expect(#LOCALE_FAILURE_MESSAGE)
                    = locale;
            }
        }.into_token_stream()
    }

    fn generate_from_key(&self, key: Key) -> Result<TokenStream, ParseError> {
        let Key { id, comment, entries } = key;

        let arguments = get_arguments(&entries[0])?;
        
        for (i, e) in entries.iter().enumerate().skip(1) {
            let a = get_arguments(e)?;
            assert!(
                a == arguments,
                "in key \"{id}\", entry {} has arguments {a:?}, but entry \
                {} has arguments {arguments:?}",
                self.locales[0].1,
                self.locales[i].1,
            );
        }

        // All go to params, but only positinal go to arguments
        let (positional, named): (Vec<_>, Vec<_>) = arguments
        .into_iter()
        .partition(|a| a.chars().all(char::is_numeric));

        let named = named
        .into_iter()
        .map(|a| syn::Ident::new(&a, Span::call_site()));

        let positional = positional
        .into_iter()
        .map(|i| format!("arg{i}"))
        .map(|a| syn::Ident::new(&a, Span::call_site()));

        let arguments = positional.clone().collect::<Vec<_>>();
        let params = named.chain(positional);

        let id = syn::Ident::new(&id, Span::call_site());
        let comment = comment.map(|c| quote! {#[doc = #c]});

        let entries = entries
        .into_iter()
        .enumerate()
        .map(|(i, entry)| {
            let locale = &self.locales[i].0;
            quote! {
                Locale::#locale => format!(#entry, #(#arguments,)*)
            }
        });

        let code = quote! {
            #comment
            pub fn #id(
                locale: Locale,
                #(#params:impl std::fmt::Display,)*
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
                    argument = format!("{unnamed_indexer}");
                    unnamed_indexer += 1;
                }
                else if !argument.starts_with(
                    |c: char| c.is_ascii_alphabetic()
                ) && !argument.chars().all(char::is_numeric)  {
                    return Err(ParseError::ArgBadStart(
                        key.to_string(), 
                        shorten(&argument), 
                        c,
                    ))
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
                    shorten(&argument),
                    c,
                ))?
            ),
            
            _ => (),
        }
    }

    Ok(arguments) 
}
