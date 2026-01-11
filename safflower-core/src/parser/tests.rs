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

fn names<const S: usize>(strs: [&str; S]) -> Vec<Name> {
    strs
    .into_iter()
    .map(Name::try_from)
    .collect::<Result<_,_>>()
    .unwrap()
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
        "",
        "locs",
        "locales",
        " locales ",
        "locales se se",
        "locales se SE",
        "locales U$",
        "locales -a",
        "locales __Temp",
    ];

    for input in ins {
        let mut configuration = Configuration::new(PathBuf::new());
        let result = configuration.parse_config(input);

        assert!(result.is_err(), "'{input}' should be err");
    }
}

#[test]
fn ok_locales() {
    let ins_outs = [
        ("locales en",         names(["en"])),
        ("locales EN",         names(["en"])),
        ("locales long-test",  names(["long_test"])),
        ("locales b-",         names(["b_"])),
        ("locales b_-",        names(["b__"])),
        ("locales se02 SE01",  names(["se02", "se01"])),
        ("locales it fr",      names(["it", "fr"])),
        ("locales \tit   fr",  names(["it", "fr"])),
    ];

    for (input, output) in ins_outs {
        let mut configuration = Configuration::new(PathBuf::new());
        let result = configuration.parse_config(input);

        assert!(result.is_ok(), "'{input}' should be ok; got {result:?}");
        assert_eq!(configuration.locales, output);
    }
}

#[test] 
fn minimal_case() {
    let tokens = vec![
        Token::Config(String::from("locales a")),
        Token::Key(name("key")),
        Token::Locale(name("a")),
        Token::Value(String::from("value")),
    ];

    let scope = parse(tokens)
    .expect("should be ok");

    assert_eq!(
        scope,
        to_scope(&[Key::name("key").entries(&["value"])]),
    );
}

#[test] 
fn key_comment() {
    let tokens = vec![
        Token::Config(String::from("locales a")),
        Token::Comment(String::from("hi!")),
        Token::Key(name("key")),
        Token::Locale(name("a")),
        Token::Value(String::from("value")),
    ];

    let scope = parse(tokens)
    .expect("should be ok");

    assert_eq!(
        scope,
        to_scope(&[
            Key::name("key").comment("hi!").entries(&["value"])
        ]),
    );
}

#[test] 
fn entry_comments() {
    let tokens = vec![
        Token::Config(String::from("locales a")),
        Token::Key(name("key")),
        Token::Comment(String::from("hi!")),
        Token::Locale(name("a")),
        Token::Value(String::from("value")),
    ];

    let scope_1 = parse(tokens)
    .expect("should be ok");

    let tokens = vec![
        Token::Config(String::from("locales a")),
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
                .entries(&["value"])
        ]),
    );
}

#[test] 
fn mutli_locales() {
    let tokens = vec![
        Token::Config(String::from("locales a b")),
        Token::Key(name("key")),
        Token::Locale(name("a")),
        Token::Value(String::from("value A")),
        Token::Locale(name("b")),
        Token::Value(String::from("value B")),
    ];

    let scope = parse(tokens)
    .expect("should be ok");

    assert_eq!(
        scope,
        to_scope(&[
            Key::name("key").entries(&["value A", "value B"])
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
        Token::Config(String::from("locales a b")),
        Token::Key(name("key")),
        Token::Locale(name("a")),
        Token::Value(String::from("value A")),
    ];

    assert!(parse(tokens).is_err());
}

#[test] 
fn using_declared_default() {
    let tokens = vec![
        Token::Config(String::from("locales a b")),
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
        Token::Config(String::from("locales b")),
        Token::Key(name("key")),
        Token::Locale(name("b")),
        Token::Value(String::from("value B")),
    ];

    assert!(parse(tokens).is_ok());
}

#[test] 
fn separate_key() {
    let tokens = vec![
        Token::Config(String::from("locales a b")),
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

    let scope = parse(tokens).expect("should be ok");

    assert_eq!(
        scope,
        to_scope(&[
            Key::name("key").entries(&["value A", "value B"]), 
            Key::name("key2").entries(&["value A", "value B"])
        ]),
    );
}

#[test]
fn single_scope() {
    let tokens = vec![
        Token::Config(String::from("locales a b")),
        
        Token::Key(name("key")),
            Token::Locale(name("a")),
            Token::Value(String::from("value A")),
            Token::Locale(name("b")),
            Token::Value(String::from("value B")),
        
        Token::Config(String::from("scope x")),
        Token::Key(name("scoped")),
            Token::Locale(name("a")),
            Token::Value(String::from("value A")),
            Token::Locale(name("b")),
            Token::Value(String::from("value B")),
    ];

    let scope = parse(tokens).expect("should be ok");

    assert_scopes_eq(
        &nest(
            to_scope(&[Key::name("key").entries(&["value A", "value B"])]),
            "x",
            to_scope(&[Key::name("scoped").entries(&["value A", "value B"])]),
        ),
        &scope,
    );
}

#[test]
fn mutliple_scope() {
    let tokens = vec![
        Token::Config(String::from("locales a b")),
        Token::Config(String::from("scope x")),
        
        Token::Key(name("key")),
            Token::Locale(name("a")),
            Token::Value(String::from("value A")),
            Token::Locale(name("b")),
            Token::Value(String::from("value B")),
        
        Token::Config(String::from("scope y")),
        Token::Key(name("scoped")),
            Token::Locale(name("a")),
            Token::Value(String::from("value A")),
            Token::Locale(name("b")),
            Token::Value(String::from("value B")),
    ];

    let scope = parse(tokens).expect("should be ok");

    let vals = &["value A", "value B"];

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
