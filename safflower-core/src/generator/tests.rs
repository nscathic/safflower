use crate::{LOCALE_FAILURE_MESSAGE, parser::Parser};

use super::*;

fn names<const S: usize>(strs: [&str; S]) -> Vec<Name> {
    strs
    .into_iter()
    .map(Name::try_from)
    .collect::<Result<_,_>>()
    .unwrap()
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
    let locales = names(["en"]);

    let generator = Generator::new(locales, Scope::default());
    let actual = generator.generate_enum();
    let expected = quote! {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        pub enum Locale { En, }
        pub const LOCALES: [Locale; 1usize] = [ Locale::En, ];
        pub static LOCALE: std::sync::Mutex<Locale> = 
                std::sync::Mutex::new(Locale::En);
    }.into_token_stream();

    assert_tokens_eq(&expected, &actual);
}

#[test]
fn enum_mutli_locales() {
    let head = names(["en", "it", "fr"]);

    let generator = Generator::new(head, Scope::default());
    let actual = generator.generate_enum();
    let expected = quote! {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        pub enum Locale {
            En,
            It,
            Fr,
        }
        pub const LOCALES: [Locale; 3usize] = [
            Locale::En,
            Locale::It,
            Locale::Fr,
        ];
        pub static LOCALE: std::sync::Mutex<Locale> = 
                std::sync::Mutex::new(Locale::En);
    }.into_token_stream();

    assert_tokens_eq(&expected, &actual);
}

#[test]
fn enum_variant_locales() {
    let locales = names(["en-US", "en_uk", "en-in"]);
    
    let generator = Generator::new(locales, Scope::default());
    let actual = generator.generate_enum();
    let expected = quote! {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        pub enum Locale {
            EnUs,
            EnUk,
            EnIn,
        }
        pub const LOCALES: [Locale; 3usize] = [
            Locale::EnUs,
            Locale::EnUk,
            Locale::EnIn,
        ];
        pub static LOCALE: std::sync::Mutex<Locale> = 
                std::sync::Mutex::new(Locale::EnUs);
    }.into_token_stream();

    assert_tokens_eq(&expected, &actual);
}

#[test]
fn single_key_single_locale() {
    let locales = names(["en"]);
    let key = Key::name("greet").comment("Common greeting.").entries(&["hi"]);
    let generator = Generator::new(locales, enscope(&[key.clone()]));
    let actual = generator.generate_from_key(key);

    let expected = quote! {
        #[doc = "Common greeting."]
        pub fn greet(locale: Locale,) -> String {
            match locale {
                Locale::En => format!("hi",),
            }
        }
    };

    assert_tokens_eq(&expected, &actual);
}

#[test]
fn single_key_single_locale_single_arg() {
    let locales = names(["en"]);
    let key = Key::name("greet") 
        .arguments(&["name"])
        .comment("Common greeting.")
        .entries(&["hi {name}"]);
    let generator = Generator::new(locales, enscope(&[key.clone()]));
    let actual = generator.generate_from_key(key);

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
    let locales = names(["en"]);
    let key = Key::name("greet") 
        .arguments(&["0", "1", "2"])
        .comment("Common greeting.")
        .entries(&["hi {0}, {1}, and {2}"]);
    let generator = Generator::new(locales, enscope(&[key.clone()]));
    let actual = generator.generate_from_key(key);

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
    let locales = names(["en", "se", "it"]);
    let key = Key::name("surprise")
        .entries(&[
            "oh my god",
            "jösses",
            "oddio",
        ]);
    let generator = Generator::new(locales, enscope(&[key.clone()]));
    let actual = generator.generate_from_key(key);

    let expected = quote! {
        pub fn surprise(locale: Locale,) -> String {
            match locale {
                Locale::En => format!("oh my god",),
                Locale::Se => format!("jösses",),
                Locale::It => format!("oddio",),
            }
        }
    };

    assert_tokens_eq(&expected, &actual);
}

#[test]
fn single_key_single_locale_generate_all() {
    let head = names(["en"]);
    let key = Key::name("greet")
        .entries(&["hi"]);
    let generator = Generator::new(head, enscope(&[key]));
    let actual = generator.generate();

    let expected = quote! {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        pub enum Locale { En, }
        pub const LOCALES: [Locale; 1usize] = [Locale::En,];
        pub static LOCALE: std::sync::Mutex<Locale> = 
                std::sync::Mutex::new(Locale::En);

        pub fn get_locale() -> Locale {
            *LOCALE
            .lock()
            .expect(#LOCALE_FAILURE_MESSAGE)
        }

        pub fn set_locale(locale: Locale) {
            *LOCALE
            .lock()
            .expect(#LOCALE_FAILURE_MESSAGE)
                = locale;
        }

        pub fn greet(locale: Locale,) -> String {
            match locale {
                Locale::En => format!("hi",),
            }
        }
    };

    assert_tokens_eq(&expected, &actual);
}

#[test]
fn multi_key_single_locale_generate_all() {
    let head = names(["en"]);
    let scope = enscope(&[
        Key::name("greet") 
            .entries(&["hi"]),
        Key::name("other_greet") 
            .entries(&["hello"]),
    ]);
    let generator = Generator::new(head, scope);
    let actual = generator.generate();

    let expected = quote! {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        pub enum Locale { En, }
        pub const LOCALES: [Locale; 1usize] = [Locale::En,];
        pub static LOCALE: std::sync::Mutex<Locale> = 
                std::sync::Mutex::new(Locale::En);

        pub fn get_locale() -> Locale {
            *LOCALE
            .lock()
            .expect(#LOCALE_FAILURE_MESSAGE)
        }

        pub fn set_locale(locale: Locale) {
            *LOCALE
            .lock()
            .expect(#LOCALE_FAILURE_MESSAGE)
                = locale;
        }

        pub fn greet(locale: Locale,) -> String {
            match locale {
                Locale::En => format!("hi",),
            }
        }

        pub fn other_greet(locale: Locale,) -> String {
            match locale {
                Locale::En => format!("hello",),
            }
        }
    };

    assert_tokens_eq(&expected, &actual);
}

#[test]
fn multi_key_multi_locale_generate_all() {
    let head = names(["en", "gr"]);
    let scope = enscope(&[
        Key::name("greet")
            .entries(&["hi", "γεια"]),
        Key::name("other_greet")
            .entries(&["hello", "καλημέρα"]),
    ]);
    let generator = Generator::new(head, scope);
    let actual = generator.generate();

    let expected = quote! {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        pub enum Locale {
            En,
            Gr,
        }
        pub const LOCALES: [Locale; 2usize] = [Locale::En, Locale::Gr,];
        pub static LOCALE: std::sync::Mutex<Locale> = 
                std::sync::Mutex::new(Locale::En);
        
        pub fn get_locale() -> Locale {
            *LOCALE
            .lock()
            .expect(#LOCALE_FAILURE_MESSAGE)
        }

        pub fn set_locale(locale: Locale) {
            *LOCALE
            .lock()
            .expect(#LOCALE_FAILURE_MESSAGE)
                = locale;
        }

        pub fn greet(locale: Locale,) -> String {
            match locale {
                Locale::En => format!("hi",),
                Locale::Gr => format!("γεια",),
            }
        }

        pub fn other_greet(locale: Locale,) -> String {
            match locale {
                Locale::En => format!("hello",),
                Locale::Gr => format!("καλημέρα",),
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

    let generator = Generator::new(parsed.locales, parsed.scope);
    let actual = generator.generate();

    let expected = quote! {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        pub enum Locale {
            En,
            Gr,
        }
        pub const LOCALES: [Locale; 2usize] = [Locale::En, Locale::Gr,];
        pub static LOCALE: std::sync::Mutex<Locale> = 
                std::sync::Mutex::new(Locale::En);

        pub fn get_locale() -> Locale {
            *LOCALE
            .lock()
            .expect(#LOCALE_FAILURE_MESSAGE)
        }

        pub fn set_locale(locale: Locale) {
            *LOCALE
            .lock()
            .expect(#LOCALE_FAILURE_MESSAGE)
                = locale;
        }

        pub fn greet(locale: Locale,) -> String {
            match locale {
                Locale::En => format!("hi",),
                Locale::Gr => format!("γεια",),
            }
        }

        pub fn other_greet(locale: Locale,) -> String {
            match locale {
                Locale::En => format!("hello",),
                Locale::Gr => format!("καλημέρα",),
            }
        }
    };

    assert_tokens_eq(&expected, &actual);
}
