use std::{ops::Not, rc::Rc};

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

use crate::parser::{Locale, Scope};

#[cfg(test)]
mod tests;

pub struct Generator;
impl Generator {
    #[must_use]
    /// Generates code.
    /// 
    /// # Errors
    /// If there are no defined locales.
    pub fn generate(
        locales: &[Rc<Locale>], 
        scope: &Scope,
    ) -> TokenStream {
        let locales = Self::generate_locales(locales);
        let getter = Self::generate_getter();
        let setter = Self::generate_setter();
        
        let keys = scope.generate_entries();

        quote! {
            #locales
            #getter
            #setter
            #keys
        }.into_token_stream()
    }
    
    /// Generates an enum of locales, and a static var to keep it.
    fn generate_locales(locales: &[Rc<Locale>]) -> TokenStream {
        let idents = locales
        .iter()
        .map(|lc| lc.ident())
        .collect::<Vec<_>>();

        let names = locales
        .iter()
        .map(|lc| lc.name())
        .collect::<Vec<_>>();

        let default = unsafe { idents.get_unchecked(0) };
        let count = locales.len();

        let enum_comment = comment("The locales available.");
        let const_comment = comment(
            "All locales, in the order they were declared."
        );
        let locale_comment = comment("The current locale.");

        quote! {
            #enum_comment
            #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
            pub enum Locale {
                #(#idents,)*
            }
            impl std::fmt::Display for Locale {
                fn fmt(
                    &self, 
                    f: &mut std::fmt::Formatter<'_>
                ) -> std::fmt::Result {
                    match self {
                        #(Self::#idents => write!(f, #names),)*
                    }
                }
            }

            #const_comment
            pub const LOCALES: [Locale; #count] = [
                #(Locale::#idents,)*
            ];

            #locale_comment
            pub static LOCALE: std::sync::RwLock<Locale> = 
                std::sync::RwLock::new(Locale::#default);
        }.into_token_stream()
    }

    /// Generates a function to get the current locale.
    fn generate_getter() -> TokenStream {
        let comment = comment("\
            Returns the current locale.\n\n\
            This blocks the thread until an exclusive write can be performed.\
        ");

        quote! {
            #comment
            pub fn get_locale() -> Locale {
                *LOCALE
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
            }
        }
    }

    /// Generates a function to set the current locale.
    fn generate_setter() -> TokenStream {
        let comment = comment("\
            Sets the current locale.\n\n\
            This blocks the thread until there is no write-lock in place; 
            multiple simultaneous reads are not blocking.\
        ");

        quote! {
            #comment
            pub fn set_locale(locale: Locale) {
                *LOCALE
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                = locale;
            }
        }.into_token_stream()
    }
}

fn comment(text: &str) -> Option<TokenStream> {
    cfg!(test)
    .not()
    .then(|| quote!{#[doc = #text]})
}
