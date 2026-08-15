#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::rc::Rc;

use crate::{name::Name, parser::{Key, Locale, Parser}};

use super::*;

fn locales<const S: usize>(strs: [[&str; 2]; S]) -> Vec<Rc<Locale>> {
    strs
    .into_iter()
    .map(|[k, n]| Rc::new(Locale::new(
        Name::try_from(k).unwrap(), 
        Some(n.into()),
    )))
    .collect::<_>()
}

fn locale(name: &str) -> Rc<Locale> {
    Rc::new(Locale::new(
        Name::try_from(name).unwrap(), 
        None,
    ))
}

fn assert_tokens_eq(expected: &TokenStream, actual: &TokenStream) {
    let expected = expected.to_string();
    let actual = actual.to_string();

    assert!(expected == actual,
        "expected != actual\n{}\nexpected: {}\n\nactual:   {}",
        colored_diff::PrettyDifference {
            expected: &expected,
            actual: &actual,
        },
        expected,
        actual,
    );
}

fn enscope(keys: &[Key]) -> Scope {
    Scope {
        keys: keys.to_vec(),
        nested: Vec::new(),
    }
}

#[test]
fn enum_single_locale() {
    let locales = locales([["en", "English"]]);
    let actual = Generator::generate_locales(&locales);

    let expected = quote! {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        pub enum Locale { En, }
        impl std::fmt::Display for Locale {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::En => write!(f, "English"),
                }
            }
        }
        pub const LOCALES: [Locale; 1usize] = [ Locale::En, ];
        pub static LOCALE: std::sync::RwLock<Locale> = 
                std::sync::RwLock::new(Locale::En);
    }.into_token_stream();

    assert_tokens_eq(&expected, &actual);
}

#[test]
fn enum_mutli_locales() {
    let locales = locales([
        ["en", "English"], 
        ["it", "Italian"], 
        ["fr", "French"],
    ]);

    let actual = Generator::generate_locales(&locales);
    let expected = quote! {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        pub enum Locale {
            En,
            It,
            Fr,
        }
        impl std::fmt::Display for Locale {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::En => write!(f, "English"),
                    Self::It => write!(f, "Italian"),
                    Self::Fr => write!(f, "French"),
                }
            }
        }
        pub const LOCALES: [Locale; 3usize] = [
            Locale::En,
            Locale::It,
            Locale::Fr,
        ];
        pub static LOCALE: std::sync::RwLock<Locale> = 
                std::sync::RwLock::new(Locale::En);
    }.into_token_stream();

    assert_tokens_eq(&expected, &actual);
}

#[test]
fn enum_variant_locales() {
    let locales = locales([
        ["en-US", "American English"], 
        ["en_uk", "British English"], 
        ["en-in", "Indian English"]
    ]);
    
    let actual = Generator::generate_locales(&locales);
    let expected = quote! {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        pub enum Locale {
            EnUs,
            EnUk,
            EnIn,
        }
        impl std::fmt::Display for Locale {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::EnUs => write!(f, "American English"),
                    Self::EnUk => write!(f, "British English"),
                    Self::EnIn => write!(f, "Indian English"),
                }
            }
        }
        pub const LOCALES: [Locale; 3usize] = [
            Locale::EnUs,
            Locale::EnUk,
            Locale::EnIn,
        ];
        pub static LOCALE: std::sync::RwLock<Locale> = 
                std::sync::RwLock::new(Locale::EnUs);
    }.into_token_stream();

    assert_tokens_eq(&expected, &actual);
}

#[test]
fn single_key_single_locale() {
    let en = locale("en");
    
    let actual = Key::name("greet")
        .comment("Common greeting.")
        .entries(&[(&en, "hi")])
        .generate();

    let expected = quote! {
        #[doc = "Common greeting."]
        pub const fn greet(locale: Locale) -> &'static str {
            match locale {
                Locale::En => "hi",
            }
        }
    };

    assert_tokens_eq(&expected, &actual);
}

#[test]
fn single_key_single_locale_single_arg() {
    let en = locale("en");
   
    let actual = Key::name("greet") 
        .arguments(&["name"])
        .comment("Common greeting.")
        .entries(&[(&en, "hi {name}")])
        .generate();

    let expected = quote! {
        #[doc = "Common greeting."]
        pub fn greet(
            locale: Locale, 
            name: impl std::fmt::Display,
        ) -> String {
            match locale {
                Locale::En => format!("hi {name}",),
            }
        }
    };

    assert_tokens_eq(&expected, &actual);
}

#[test]
fn single_key_single_locale_multi_arg() {
    let en = locale("en");

    let actual = Key::name("greet") 
        .arguments(&["0", "1", "2"])
        .comment("Common greeting.")
        .entries(&[(&en, "hi {0}, {1}, and {2}")])
        .generate();

    let expected = quote! {
        #[doc = "Common greeting."]
        pub fn greet(
            locale: Locale, 
            arg0: impl std::fmt::Display,
            arg1: impl std::fmt::Display,
            arg2: impl std::fmt::Display,
        ) -> String {
            match locale {
                Locale::En => format!("hi {0}, {1}, and {2}", arg0, arg1, arg2,),
            }
        }
    };

    assert_tokens_eq(&expected, &actual);
}

#[test]
fn single_key_mutli_locale() {
    let en = locale("en");
    let se = locale("se");
    let it = locale("it");

    let actual = Key::name("surprise")
        .entries(&[
            (&en, "oh my god"),
            (&se, "jösses"),
            (&it, "oddio"),
        ])
        .generate();

    let expected = quote! {
        pub const fn surprise(locale: Locale) -> &'static str {
            match locale {
                Locale::En => "oh my god",
                Locale::Se => "jösses",
                Locale::It => "oddio",
            }
        }
    };

    assert_tokens_eq(&expected, &actual);
}

#[test]
fn single_key_single_locale_generate_all() {
    let locales = locales([["en", "English"]]);
    let en = &locales[0];

    let key = Key::name("greet")
        .entries(&[(en, "hi")]);
    let scope = enscope(std::slice::from_ref(&key));

    let actual = Generator::generate(&locales, &scope);

    let expected = quote! {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        pub enum Locale { En, }
        impl std::fmt::Display for Locale {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::En => write!(f, "English"),
                }
            }
        }
        pub const LOCALES: [Locale; 1usize] = [Locale::En,];
        pub static LOCALE: std::sync::RwLock<Locale> = 
                std::sync::RwLock::new(Locale::En);

        pub fn get_locale() -> Locale {
            *LOCALE
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
        }

        pub fn set_locale(locale: Locale) {
            *LOCALE
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            = locale;
        }

        pub const fn greet(locale: Locale) -> &'static str {
            match locale {
                Locale::En => "hi",
            }
        }
    };

    assert_tokens_eq(&expected, &actual);
}

#[test]
fn multi_key_single_locale_generate_all() {
    let locales = locales([["en", "English"]]);
    let en = &locales[0];

    let scope = enscope(&[
        Key::name("greet") 
            .entries(&[(en, "hi")]),
        Key::name("other_greet") 
            .entries(&[(en, "hello")]),
    ]);

    let actual = Generator::generate(&locales, &scope);

    let expected = quote! {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        pub enum Locale { En, }
        impl std::fmt::Display for Locale {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::En => write!(f, "English"),
                }
            }
        }
        pub const LOCALES: [Locale; 1usize] = [Locale::En,];
        pub static LOCALE: std::sync::RwLock<Locale> = 
                std::sync::RwLock::new(Locale::En);

        pub fn get_locale() -> Locale {
            *LOCALE
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
        }

        pub fn set_locale(locale: Locale) {
            *LOCALE
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            = locale;
        }

        pub const fn greet(locale: Locale) -> &'static str {
            match locale {
                Locale::En => "hi",
            }
        }

        pub const fn other_greet(locale: Locale) -> &'static str {
            match locale {
                Locale::En => "hello",
            }
        }
    };

    assert_tokens_eq(&expected, &actual);
}

#[test]
fn multi_key_multi_locale_generate_all() {
    let locales = locales([
        ["en", "English"],
        ["gr", "Greek"],
    ]);
    let en = &locales[0];
    let gr = &locales[1];

    let scope = enscope(&[
        Key::name("greet")
            .entries(&[(en,"hi"), (gr, "γεια")]),
        Key::name("other_greet")
            .entries(&[(en,"hello"), (gr, "καλημέρα")]),
    ]);
    let actual = Generator::generate(&locales, &scope);

    let expected = quote! {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        pub enum Locale {
            En,
            Gr,
        }
        impl std::fmt::Display for Locale {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::En => write!(f, "English"),
                    Self::Gr => write!(f, "Greek"),
                }
            }
        }
        pub const LOCALES: [Locale; 2usize] = [Locale::En, Locale::Gr,];
        pub static LOCALE: std::sync::RwLock<Locale> = 
                std::sync::RwLock::new(Locale::En);
        
        pub fn get_locale() -> Locale {
            *LOCALE
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
        }

        pub fn set_locale(locale: Locale) {
            *LOCALE
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            = locale;
        }

        pub const fn greet(locale: Locale) -> &'static str {
            match locale {
                Locale::En => "hi",
                Locale::Gr => "γεια",
            }
        }

        pub const fn other_greet(locale: Locale) -> &'static str {
            match locale {
                Locale::En => "hello",
                Locale::Gr => "καλημέρα",
            }
        }
    };

    assert_tokens_eq(&expected, &actual);
}

#[test]
fn multi_from_text() {
    let source = "
        !locales en gr
        greet:
            en \"hi\"
            gr \"γεια\"
        other_greet:
            en \"hello\"
            gr \"καλημέρα\"
    ";
    let parsed = Parser::from_text(source).parse().unwrap();

    let actual = Generator::generate(
        &parsed.locales, 
        &parsed.scope,
    );

    let expected = quote! {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        pub enum Locale {
            En,
            Gr,
        }
        impl std::fmt::Display for Locale {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::En => write!(f, "en"),
                    Self::Gr => write!(f, "gr"),
                }
            }
        }
        pub const LOCALES: [Locale; 2usize] = [Locale::En, Locale::Gr,];
        pub static LOCALE: std::sync::RwLock<Locale> = 
                std::sync::RwLock::new(Locale::En);

        pub fn get_locale() -> Locale {
            *LOCALE
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
        }

        pub fn set_locale(locale: Locale) {
            *LOCALE
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            = locale;
        }

        pub const fn greet(locale: Locale) -> &'static str {
            match locale {
                Locale::En => "hi",
                Locale::Gr => "γεια",
            }
        }

        pub const fn other_greet(locale: Locale) -> &'static str {
            match locale {
                Locale::En => "hello",
                Locale::Gr => "καλημέρα",
            }
        }
    };

    assert_tokens_eq(&expected, &actual);
}
