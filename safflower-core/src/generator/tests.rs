use crate::{parser::Parser, reader::CharReader};

use super::*;

#[test]
fn no_locales() {
    let head = Head::default();
    let generator = Generator::new(head, Vec::new());

    assert!(generator.generate_getter().is_err());

    let expected = quote! {
        pub enum Locale {}
    }.into_token_stream();
    assert_tokens_eq(&expected, &generator.generate_enum());
}

#[test]
fn single_locale() {
    let head = get_head(&["en"]);

    let generator = Generator::new(head, Vec::new());
    let actual = generator.generate_enum();
    let expected = quote! {
        pub enum Locale {
            En,
        }
    }.into_token_stream();

    assert_tokens_eq(&expected, &actual);
}

#[test]
fn mutli_locales() {
    let head = get_head(&["en", "it", "fr"]);

    let generator = Generator::new(head, Vec::new());
    let actual = generator.generate_enum();
    let expected = quote! {
        pub enum Locale {
            En,
            It,
            Fr,
        }
    }.into_token_stream();

    assert_tokens_eq(&expected, &actual);
}

#[test]
fn variant_locales() {
    let head = get_head(&["en_us", "en_uk", "en_in"]);
    
    let generator = Generator::new(head, Vec::new());
    let actual = generator.generate_enum();
    let expected = quote! {
        pub enum Locale {
            EnUs,
            EnUk,
            EnIn,
        }
    }.into_token_stream();

    assert_tokens_eq(&expected, &actual);
}

#[test]
fn single_locale_getter() {
    let head = get_head(&["omega"]);
    let generator = Generator::new(head, vec![]);
    let actual = generator.generate_getter().unwrap();
    
    let expected = quote! {
        pub fn get_locale() -> Locale {
            let Ok(var) = std::env::var(#ENV_LOCALE_NAME) else {
                return Locale::Omega;
            };

            match var.as_str() {
                _ => Locale::Omega,
            }
        }
    }.into_token_stream();

    assert_tokens_eq(&expected, &actual);
}

#[test]
fn mutli_locale_getter() {
    let head = get_head(&["en-uk", "se", "it"]);
    let generator = Generator::new(head, vec![]);
    let actual = generator.generate_getter().unwrap();
    
    let expected = quote! {
        pub fn get_locale() -> Locale {
            let Ok(var) = std::env::var(#ENV_LOCALE_NAME) else {
                return Locale::EnUk;
            };

            match var.as_str() {
                "SE" => Locale::Se,
                "IT" => Locale::It,
                _ => Locale::EnUk,
            }
        }
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
        pub fn greet(locale: Locale) -> String {
            match locale {
                Locale::En => format!("hi"),
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
        pub fn surprise(locale: Locale) -> String {
            match locale {
                Locale::En => format!("oh my god"),
                Locale::Se => format!("jösses"),
                Locale::It => format!("oddio"),
            }
        }
    };

    assert_tokens_eq(&expected, &actual);
}

#[test]
fn single_key_single_locale_complete() {
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
        pub enum Locale {
            En,
        }
        pub fn get_locale() -> Locale {
            let Ok(var) = std::env::var(#ENV_LOCALE_NAME) else {
                return Locale::En;
            };

            match var.as_str() {
                _ => Locale::En,
            }
        }
        pub fn greet(locale: Locale) -> String {
            match locale {
                Locale::En => format!("hi"),
            }
        }
    };

    assert_tokens_eq(&expected, &actual);
}

#[test]
fn multi_key_single_locale() {
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
        pub enum Locale {
            En,
        }
        pub fn get_locale() -> Locale {
            let Ok(var) = std::env::var(#ENV_LOCALE_NAME) else {
                return Locale::En;
            };

            match var.as_str() {
                _ => Locale::En,
            }
        }

        pub fn greet(locale: Locale) -> String {
            match locale {
                Locale::En => format!("hi"),
            }
        }

        pub fn other_greet(locale: Locale) -> String {
            match locale {
                Locale::En => format!("hello"),
            }
        }
    };

    assert_tokens_eq(&expected, &actual);
}

#[test]
fn multi_key_multi_locale() {
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
        pub enum Locale {
            En,
            Gr,
        }
        pub fn get_locale() -> Locale {
            let Ok(var) = std::env::var(#ENV_LOCALE_NAME) else {
                return Locale::En;
            };

            match var.as_str() {
                "GR" => Locale::Gr,
                _ => Locale::En,
            }
        }

        pub fn greet(locale: Locale) -> String {
            match locale {
                Locale::En => format!("hi"),
                Locale::Gr => format!("γεια"),
            }
        }

        pub fn other_greet(locale: Locale) -> String {
            match locale {
                Locale::En => format!("hello"),
                Locale::Gr => format!("καλημέρα"),
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
        pub enum Locale {
            En,
            Gr,
        }
        pub fn get_locale() -> Locale {
            let Ok(var) = std::env::var(#ENV_LOCALE_NAME) else {
                return Locale::En;
            };

            match var.as_str() {
                "GR" => Locale::Gr,
                _ => Locale::En,
            }
        }

        pub fn greet(locale: Locale) -> String {
            match locale {
                Locale::En => format!("hi"),
                Locale::Gr => format!("γεια"),
            }
        }

        pub fn other_greet(locale: Locale) -> String {
            match locale {
                Locale::En => format!("hello"),
                Locale::Gr => format!("καλημέρα"),
            }
        }
    };

    assert_tokens_eq(&expected, &actual);
}

#[test]
fn no_arguments() {
    for line in [
        "Hello",
        "",
    ] {
        let result = get_arguments(line).unwrap();
        assert!(result.is_empty());
    }
}

#[test]
fn invalid_arguments() {
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
fn single_arguments() {
    for (line, arg) in [
        ("Hello {name}", "name"),
        ("{0} is really cool", "arg0"),
        ("{arg-b}", "arg_b"),
        ("{}", "arg0"),
    ] {
        let result = get_arguments(line).unwrap();
        assert_eq!(result, vec![arg]);
    }
}

#[test]
fn mutliple_arguments() {
    for (line, arg) in [
        ("Hello {name}, I'm {name2}", vec!["name", "name2"]),
        ("{0}{1}{3}", vec!["arg0", "arg1", "arg3"]),
        ("{}{}{}", vec!["arg0", "arg1", "arg2"]),
    ] {
        let result = get_arguments(line).unwrap();
        assert_eq!(result, arg);
    }
}

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
