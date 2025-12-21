use crate::{LOCALE_FAILURE_MESSAGE, parser::Parser, reader::CharReader};

use super::*;

fn get_head(locales: &[&str]) -> Head {
    let mut head = Head::default();
    head.set_locales(locales).unwrap();
    head
}

fn assert_tokens_eq(expected: &TokenStream, actual: &TokenStream) {
    let expected = expected.to_string();
    let actual = actual.to_string();

    if expected != actual {
        panic!(
            "expected != actual\n{}\nexpected: {}\nactual:   {}",
            colored_diff::PrettyDifference {
                expected: &expected,
                actual: &actual,
            },
            expected,
            actual,
        );
    }
}


#[test]
fn enum_no_locales() {
    let head = Head::default();
    let generator = Generator::new(head, Vec::new());
    assert!(generator.generate_enum().is_err());
}

#[test]
fn enum_single_locale() {
    let head = get_head(&["en"]);

    let generator = Generator::new(head, Vec::new());
    let actual = generator.generate_enum().unwrap();
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
    let head = get_head(&["en", "it", "fr"]);

    let generator = Generator::new(head, Vec::new());
    let actual = generator.generate_enum().unwrap();
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
    let head = get_head(&["en-US", "en_uk", "en-in"]);
    
    let generator = Generator::new(head, Vec::new());
    let actual = generator.generate_enum().unwrap();
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
    let head = get_head(&["en"]);
    let key = Key { 
        id: String::from("greet"), 
        comment: Some(String::from("Common greeting.")),
        entries: vec![
            String::from("hi"),
        ]
    };
    let generator = Generator::new(head, vec![key.clone()]);
    let actual = generator.generate_from_key(key).unwrap();

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
    let head = get_head(&["en"]);
    let key = Key { 
        id: String::from("greet"), 
        comment: Some(String::from("Common greeting.")),
        entries: vec![
            String::from("hi {name}"),
        ]
    };
    let generator = Generator::new(head, vec![key.clone()]);
    let actual = generator.generate_from_key(key).unwrap();

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
    let head = get_head(&["en"]);
    let key = Key { 
        id: String::from("greet"), 
        comment: Some(String::from("Common greeting.")),
        entries: vec![
            String::from("hi {0}, {1}, and {2}"),
        ]
    };
    let generator = Generator::new(head, vec![key.clone()]);
    let actual = generator.generate_from_key(key).unwrap();

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
    let head = get_head(&["en", "se", "it"]);
    let key = Key { 
        id: String::from("surprise"), 
        comment: None, 
        entries: vec![
            String::from("oh my god"),
            String::from("jösses"),
            String::from("oddio"),
        ]
    };
    let generator = Generator::new(head, vec![key.clone()]);
    let actual = generator.generate_from_key(key).unwrap();

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
fn parse_no_arguments() {
    for line in [
        "Hello",
        "",
    ] {
        let result = get_arguments(line).unwrap();
        assert!(result.is_empty());
    }
}

#[test]
fn parse_invalid_arguments() {
    for line in [
        "Hi {$arg}",
        "I want a {{}",
        "Do you want a {}}?",
        "No, but a {?}",
    ] {
        let result = get_arguments(line);
        assert!(
            result.is_err(), 
            "{line} should fault, is instead {:?}", 
            result.unwrap(),
        );
    }
}

#[test]
fn parse_single_arguments() {
    for (line, arg) in [
        ("Hello {name}", "name"),
        ("{0} is really cool", "0"),
        ("{arg-b}", "arg_b"),
        ("{}", "0"),
    ] {
        let result = get_arguments(line).unwrap();
        assert_eq!(result, vec![arg]);
    }
}

#[test]
fn parse_mutliple_arguments() {
    for (line, arg) in [
        ("Hello {name}, I'm {name2}", vec!["name", "name2"]),
        ("{0}{1}{3}", vec!["0", "1", "3"]),
        ("{}{}{}", vec!["0", "1", "2"]),
    ] {
        let result = get_arguments(line).unwrap();
        assert_eq!(result, arg);
    }
}

#[test]
fn single_key_single_locale_generate_all() {
    let head = get_head(&["en"]);
    let key = Key { 
        id: String::from("greet"), 
        comment: None, 
        entries: vec![
            String::from("hi"),
        ]
    };
    let generator = Generator::new(head, vec![key]);
    let actual = generator.generate().unwrap();

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
    let head = get_head(&["en"]);
    let keys = vec![
        Key { 
            id: String::from("greet"), 
            comment: None, 
            entries: vec![
                String::from("hi"),
            ]
        },
        Key { 
            id: String::from("other_greet"), 
            comment: None, 
            entries: vec![
                String::from("hello"),
            ]
        },
    ];
    let generator = Generator::new(head, keys);
    let actual = generator.generate().unwrap();

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
    let head = get_head(&["en", "gr"]);
    let keys = vec![
        Key { 
            id: String::from("greet"), 
            comment: None, 
            entries: vec![
                String::from("hi"),
                String::from("γεια"),
            ]
        },
        Key { 
            id: String::from("other_greet"), 
            comment: None, 
            entries: vec![
                String::from("hello"),
                String::from("καλημέρα"),
            ]
        },
    ];
    let generator = Generator::new(head, keys);
    let actual = generator.generate().unwrap();

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
    let reader = CharReader::new(source);
    let mut parser = Parser::default();
    parser.parse(reader).unwrap();
    let (head, keys) = parser.extract();
    let generator = Generator::new(head, keys);
    let actual = generator.generate().unwrap();

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
