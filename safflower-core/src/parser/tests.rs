use crate::{name::Name, reader::Token};
use super::*;

fn parse_tokens(tokens: Vec<Token>) -> Result<Vec<Key>, Error> {
    let mut parser = Parser::default();
    parser.parse(tokens.into_iter().map(Ok))?;
    parser.collect().map(|pd| pd.keys).map_err(Into::into)
}

fn name(str: &str) -> Name { Name::try_from(str).unwrap() }

fn names<const S: usize>(strs: [&str; S]) -> Vec<Name> {
    strs
    .into_iter()
    .map(Name::try_from)
    .collect::<Result<_,_>>()
    .unwrap()
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
        let mut configuration = Configuration::default();
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
        let mut configuration = Configuration::default();
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

    let keys = parse_tokens(tokens)
    .expect("should be ok");

    assert_eq!(
        keys,
        vec![
            Key { 
                id: name("key"), 
                arguments: vec![],
                comment: None, 
                entries: vec![String::from("value")] 
            }
        ]
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

    let keys = parse_tokens(tokens)
    .expect("should be ok");

    assert_eq!(
        keys,
        vec![
            Key { 
                id: name("key"), 
                arguments: vec![],
                comment: Some(String::from("hi!")), 
                entries: vec![String::from("value")] 
            }
        ]
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

    let keys_1 = parse_tokens(tokens)
    .expect("should be ok");

    let tokens = vec![
        Token::Config(String::from("locales a")),
        Token::Key(name("key")),
        Token::Locale(name("a")),
        Token::Comment(String::from("hi!")),
        Token::Value(String::from("value")),
    ];

    let keys_2 = parse_tokens(tokens)
    .expect("should be ok");

    assert_eq!(keys_1, keys_2);

    assert_eq!(
        keys_1,
        vec![
            Key { 
                id: name("key"), 
                arguments: vec![],
                comment: Some(String::from(" # Locale notes\n- *a*: hi!\n")), 
                entries: vec![String::from("value")] 
            }
        ]
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

    let keys = parse_tokens(tokens)
    .expect("should be ok");

    assert_eq!(
        keys,
        vec![
            Key { 
                id: name("key"), 
                arguments: vec![],
                comment: None, 
                entries: vec![
                    String::from("value A"),
                    String::from("value B"), 
                ] 
            }
        ]
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

    assert!(parse_tokens(tokens).is_err());
}

#[test] 
fn missing_declared_locale() {
    let tokens = vec![
        Token::Config(String::from("locales a b")),
        Token::Key(name("key")),
        Token::Locale(name("a")),
        Token::Value(String::from("value A")),
    ];

    assert!(parse_tokens(tokens).is_err());
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

    assert!(parse_tokens(tokens).is_err());
}

#[test] 
fn using_and_not_default() {
    let tokens = vec![
        Token::Key(name("key")),
        Token::Locale(name("b")),
        Token::Value(String::from("value B")),
    ];

    assert!(parse_tokens(tokens).is_err());

    let tokens = vec![
        Token::Config(String::from("locales b")),
        Token::Key(name("key")),
        Token::Locale(name("b")),
        Token::Value(String::from("value B")),
    ];

    assert!(parse_tokens(tokens).is_ok());
}

#[test]
fn parse_no_arguments() {
    for line in [
        "Hello",
        "",
    ] {
        let result = extract_arguments(line).unwrap();
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
        let result = extract_arguments(line);
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
        let result = extract_arguments(line).unwrap();
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
        let result = extract_arguments(line).unwrap();
        assert_eq!(result, arg);
    }
}
