use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};

use crate::{LOCALE_FAILURE_MESSAGE, name::Name, parser::Key};

#[cfg(test)]
mod tests;

pub struct Generator {
    locales: Vec<(syn::Ident, String)>,
    keys: Vec<Key>,
}

impl Generator {
    #[must_use] 
    /// Sets itself up.
    pub fn new(locales: Vec<Name>, keys: Vec<Key>) -> Self {
        let locales = locales
        .into_iter()
        .map(|loc| (
            syn::Ident::new(&loc.type_name(), Span::call_site()),
            loc.into(), 
        ))
        .collect();

        Self { 
            locales, 
            keys,
        }
    }

    #[must_use]
    /// Generates code.
    /// 
    /// # Errors
    /// If there are no defined locales.
    pub fn generate(mut self) -> TokenStream {
        let locales = self.generate_enum();
        let getter = Self::generate_getter();
        let setter = Self::generate_setter();
        
        let keys = std::mem::take(&mut self.keys)
        .into_iter()
        .map(|key| self.generate_from_key(key))
        .collect::<Vec<_>>();

        quote! {
            #locales
            #getter
            #setter
            #(#keys)*
        }.into_token_stream()
    }
    
    /// Generates an enum of locales, and a static var to keep it.
    fn generate_enum(&self) -> TokenStream {
        let locales = self.locales.iter().map(|(i, _)| i).collect::<Vec<_>>();
        let default = locales[0];
        let count = self.locales.len();

        let enum_comment = comment("The locales available.");
        let const_comment = comment("All locales, in the order they were \
            declared.");
        let locale_comment = comment("The current locale.");

        quote! {
            #enum_comment
            #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
            pub enum Locale {
                #(#locales,)*
            }

            #const_comment
            pub const LOCALES: [Locale; #count] = [
                #(Locale::#locales,)*
            ];

            #locale_comment
            pub static LOCALE: std::sync::Mutex<Locale> = 
                std::sync::Mutex::new(Locale::#default);
        }.into_token_stream()
    }

    /// Generates a function to get the current locale.
    fn generate_getter() -> TokenStream {
        let comment = comment("\
            Returns the current locale. As this calls `Mutex::lock()`, it \
            will block the thread until it is safe to access. \n\n\
            # Panic \n\
            It will panic if the `Mutex` has been poisoned. See \
            [`std::sync::Mutex`].");

        quote! {
            #comment
            pub fn get_locale() -> Locale {
                *LOCALE
                .lock()
                .expect(#LOCALE_FAILURE_MESSAGE)
            }
        }
    }

    /// Generates a function to set the current locale.
    fn generate_setter() -> TokenStream {
        let comment = comment("\
            Sets the current locale. As this calls `Mutex::lock()`, it will \
            block the thread until it is safe to access. \n\n\
            # Panic \n\
            It will panic if the `Mutex` has been poisoned. See \
            [`std::sync::Mutex`].");

        quote! {
            #comment
            pub fn set_locale(locale: Locale) {
                *LOCALE
                .lock()
                .expect(#LOCALE_FAILURE_MESSAGE)
                    = locale;
            }
        }.into_token_stream()
    }

    fn generate_from_key(&self, key: Key) -> TokenStream {
        let Key { id, arguments, comment, entries } = key;

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

        let id = syn::Ident::new(id.to_str(), Span::call_site());
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

        quote! {
            #comment
            pub fn #id(
                locale: Locale,
                #(#params:impl std::fmt::Display,)*
            ) -> String {
                match locale {
                    #(#entries,)*
                }
            }
        }
    }
}

fn comment(text: &str) -> Option<TokenStream> {
    if cfg!(test) {
        None
    } else {
        Some(quote!{#[doc = #text]})
    }
}
