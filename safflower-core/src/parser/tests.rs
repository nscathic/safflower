#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use std::vec;

use crate::{name::Name, reader::Token};
use super::*;

fn parse(tokens: Vec<Token>) -> Result<Scope, Error> {
    Parser::from_vec(tokens)
    .parse()
    .map(|pd| pd.scope)
}

fn to_scope(keys: &[Key]) -> Scope { 
    Scope { keys: keys.to_vec(), nested: Vec::new() }
}

fn nest(mut parent: Scope, str: &str, child: Scope) -> Scope {
    parent.nested.push((name(str), Box::new(child)));
    parent
}

fn name(str: &str) -> Name { Name::try_from(str).unwrap() }

fn locale(name: &str) -> Rc<Locale> {
    Rc::new(Locale::new( 
        Name::try_from(name).unwrap(), 
        None,
    ))
}

fn assert_scopes_eq(expected: &Scope, actual: &Scope) {
    let expected = format!("{expected:?}");
    let actual = format!("{actual:?}");

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

#[test]
fn bad_locales() {
    let ins = [
        ("", ""),
        ("locs", ""),
        ("locales", ""),
        (" locales ", ""),
        ("locales", "se se"),
        ("locales", "se SE"),
        ("locales", "U$",),
        ("locales", "-a"),
        ("locales", "__Temp"),
    ];

    for (key, args) in ins {
        let mut configuration = Configuration::new(PathBuf::new());
        let result = configuration.parse_config(key, args);

        assert!(result.is_err(), "('{key}' '{args}') should be err");
    }
}

#[test]
fn ok_locales() {
    let ins_outs = [
        ("locales", "en",         vec!["en"]),
        ("locales", "EN",         vec!["en"]),
        ("locales", "long-test",  vec!["long_test"]),
        ("locales", "b-",         vec!["b_"]),
        ("locales", "b_-",        vec!["b__"]),
        ("locales", "se02 SE01",  vec!["se02", "se01"]),
        ("locales", "it fr",      vec!["it", "fr"]),
        ("locales", "\tit   fr",  vec!["it", "fr"]),
    ];

    for (key, args, output) in ins_outs {
        let mut configuration = Configuration::new(PathBuf::new());
        let result = configuration.parse_config(key, args);

        assert!(
            result.is_ok(),
            "('{key}' '{args}') should be ok; got {result:?}",
        );
        
        let locs = configuration.into_locales();
        let actual = locs
        .iter()
        .map(|lc| lc.key().as_str())
        .collect::<Vec<_>>();
    
        assert_eq!(actual, output);
    }
}

#[test]
fn ok_locale_names() {
    let ins_outs = [
        ("locales", "en(Eng)", 
            vec![("en", "Eng")]),
        ("locales", "EN(eng)", 
            vec![("en", "eng")]),
        ("locales", "long-test(Long Test)", 
            vec![("long_test", "Long Test")]),
        ("locales", "b-( b )", 
            vec![("b_", " b ")]),
        ("locales", "it(Italian) fr(European French)", 
            vec![("it", "Italian"), ("fr", "European French")]),
    ];

    for (key, args, output) in ins_outs {
        let mut configuration = Configuration::new(PathBuf::new());
        let result = configuration.parse_config(key, args);

        assert!(
            result.is_ok(), 
            "('{key}' '{args}') should be ok; got {result:?}",
        );

        configuration
        .into_locales()
        .into_iter()
        .enumerate()
        .for_each(|(i, lc)| {
            assert_eq!(
                lc.key().as_str(),
                output[i].0,
                "key mismatch"
            );
            assert_eq!(
                lc.name(),
                output[i].1,
                "name mismatch"
            );
        });
    }
}

#[test]
fn bad_locale_names() {
    let ins = [
        ("locales", "en("),
        ("locales", "EN (eng)"), 
        ("locales", "long-test(Long Test"), 
        ("locales", "b-((b))"), 
        ("locales", "it(Italian) fr(European French))"), 
    ];

    for (key, args) in ins {
        let mut configuration = Configuration::new(PathBuf::new());
        let result = configuration.parse_config(key, args);

        assert!(
            result.is_err(), 
            "('{key}' '{args}') should be err",
        );
    }
}

#[test] 
fn minimal_case() {
    let tokens = vec![
        Token::Config("locales".into(), "a".into()),
        Token::Key(name("key")),
        Token::Locale(name("a")),
        Token::Value(String::from("value")),
    ];

    let scope = parse(tokens)
    .expect("should be ok");

    let loc = locale("a");

    assert_eq!(
        scope,
        to_scope(&[Key::name("key").entries(&[(&loc, "value")])]),
    );
}

#[test] 
fn key_comment() {
    let tokens = vec![
        Token::Config("locales".into(), "a".into()),
        Token::Comment(String::from("hi!")),
        Token::Key(name("key")),
        Token::Locale(name("a")),
        Token::Value(String::from("value")),
    ];

    let scope = parse(tokens)
    .expect("should be ok");

    let loc = locale("a");

    assert_eq!(
        scope,
        to_scope(&[
            Key::name("key").comment("hi!").entries(&[(&loc, "value")])
        ]),
    );
}

#[test] 
fn entry_comments() {
    let tokens = vec![
        Token::Config("locales".into(), "a".into()),
        Token::Key(name("key")),
        Token::Comment(String::from("hi!")),
        Token::Locale(name("a")),
        Token::Value(String::from("value")),
    ];
    let loc = locale("a");

    let scope_1 = parse(tokens)
    .expect("should be ok");

    let tokens = vec![
        Token::Config("locales".into(), "a".into()),
        Token::Key(name("key")),
        Token::Locale(name("a")),
        Token::Comment(String::from("hi!")),
        Token::Value(String::from("value")),
    ];

    let scope_2 = parse(tokens)
    .expect("should be ok");

    assert_eq!(scope_1, scope_2);

    assert_eq!(
        scope_1,
        to_scope(&[
            Key::name("key")
                .comment(" # Locale notes\n- *a*: hi!\n")
                .entries(&[(&loc, "value")])
        ]),
    );
}

#[test] 
fn mutli_locales() {
    let tokens = vec![
        Token::Config("locales".into(), "a b".into()),
        Token::Key(name("key")),
        Token::Locale(name("a")),
        Token::Value(String::from("value A")),
        Token::Locale(name("b")),
        Token::Value(String::from("value B")),
    ];
    let a = locale("a");
    let b = locale("b");

    let scope = parse(tokens)
    .expect("should be ok");

    assert_eq!(
        scope,
        to_scope(&[
            Key::name("key").entries(&[
                (&a, "value A"), 
                (&b, "value B"),
            ])
        ]),
    );
}

#[test] 
fn missing_locales() {
    let tokens = vec![
        Token::Key(name("key")),
        Token::Locale(name("a")),
        Token::Value(String::from("value A")),
        Token::Locale(name("b")),
        Token::Value(String::from("value B")),
    ];

    assert!(parse(tokens).is_err());
}

#[test] 
fn missing_declared_locale() {
    let tokens = vec![
        Token::Config("locales".into(), "a b".into()),
        Token::Key(name("key")),
        Token::Locale(name("a")),
        Token::Value(String::from("value A")),
    ];

    assert!(parse(tokens).is_err());
}

#[test] 
fn using_declared_default() {
    let tokens = vec![
        Token::Config("locales".into(), "a b".into()),
        Token::Key(name("key")),
        Token::Value(String::from("value A")),
        Token::Locale(name("b")),
        Token::Value(String::from("value B")),
    ];

    assert!(parse(tokens).is_err());
}

#[test] 
fn using_and_not_default() {
    let tokens = vec![
        Token::Key(name("key")),
        Token::Locale(name("b")),
        Token::Value(String::from("value B")),
    ];

    assert!(parse(tokens).is_err());

    let tokens = vec![
        Token::Config("locales".into(), "b".into()),
        Token::Key(name("key")),
        Token::Locale(name("b")),
        Token::Value(String::from("value B")),
    ];

    assert!(parse(tokens).is_ok());
}

#[test] 
fn separate_key() {
    let tokens = vec![
        Token::Config("locales".into(), "a b".into()),
        Token::Key(name("key")),
        Token::Locale(name("a")),
        Token::Value(String::from("value A")),
        Token::Key(name("key2")),
        Token::Locale(name("a")),
        Token::Value(String::from("value A")),
        Token::Key(name("key")),
        Token::Locale(name("b")),
        Token::Value(String::from("value B")),
        Token::Key(name("key2")),
        Token::Locale(name("b")),
        Token::Value(String::from("value B")),
    ];
    let a = locale("a");
    let b = locale("b");

    let scope = parse(tokens).expect("should be ok");

    assert_eq!(
        scope,
        to_scope(&[
            Key::name("key").entries(&[
                (&a, "value A"), 
                (&b, "value B"),
            ]), 
            Key::name("key2").entries(&[
                (&a, "value A"), 
                (&b, "value B"),
            ])
        ]),
    );
}

#[test]
fn single_scope() {
    let tokens = vec![
        Token::Config("locales".into(), "a b".into()),
        
        Token::Key(name("key")),
            Token::Locale(name("a")),
            Token::Value(String::from("value A")),
            Token::Locale(name("b")),
            Token::Value(String::from("value B")),
        
        Token::Config("scope".into(), "x".into()),
        Token::Key(name("scoped")),
            Token::Locale(name("a")),
            Token::Value(String::from("value A")),
            Token::Locale(name("b")),
            Token::Value(String::from("value B")),
    ];
    let a = locale("a");
    let b = locale("b");

    let scope = parse(tokens).expect("should be ok");

    assert_scopes_eq(
        &nest(
            to_scope(&[Key::name("key").entries(&[
                (&a, "value A"), 
                (&b, "value B"),
            ])]),
            "x",
            to_scope(&[Key::name("scoped").entries(&[
                (&a, "value A"), 
                (&b, "value B"),
            ])]),
        ),
        &scope,
    );
}

#[test]
fn mutliple_scope() {
    let tokens = vec![
        Token::Config("locales".into(), "a b".into()),
        Token::Config("scope".into(), "x".into()),
        
        Token::Key(name("key")),
            Token::Locale(name("a")),
            Token::Value(String::from("value A")),
            Token::Locale(name("b")),
            Token::Value(String::from("value B")),
        
        Token::Config("scope".into(), "y".into()),
        Token::Key(name("scoped")),
            Token::Locale(name("a")),
            Token::Value(String::from("value A")),
            Token::Locale(name("b")),
            Token::Value(String::from("value B")),
    ];
    let a = locale("a");
    let b = locale("b");

    let scope = parse(tokens).expect("should be ok");

    let vals = &[(&a, "value A"), (&b, "value B")];

    assert_scopes_eq(
        &nest(
            nest(
                to_scope(&[]), 
                "x", 
                to_scope(&[Key::name("key").entries(vals)]),
            ),
            "y", 
            to_scope(&[Key::name("scoped").entries(vals)]),
        ),
        &scope,
    );
}
