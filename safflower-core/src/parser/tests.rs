use crate::{
    error::ParseError, parser::{Key, Parser, Head}, reader::Token
};

fn parse_tokens(tokens: Vec<Token>) -> Result<Vec<Key>, ParseError> {
    let mut parser = Parser::default();
    parser.parse(tokens.into_iter())?;
    Ok(parser.keys)
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
        let mut head = Head::default();
        let result = head.parse_config(input);

        assert!(result.is_err(), "'{input}' should be err");
    }
}

#[test]
fn ok_locales() {
    let ins_outs = [
        ("locales en",         vec!["en"]),
        ("locales EN",         vec!["en"]),
        ("locales long-test",  vec!["long_test"]),
        ("locales b-",         vec!["b_"]),
        ("locales b_-",        vec!["b__"]),
        ("locales se02 SE01",  vec!["se02", "se01"]),
        ("locales it fr",      vec!["it", "fr"]),
        ("locales \tit   fr",  vec!["it", "fr"]),
    ];

    for (input, output) in ins_outs {
        let mut head = Head::default();
        let result = head.parse_config(input);

        assert!(result.is_ok(), "'{input}' should be ok; got {result:?}");
        assert_eq!(head.locales, output);
    }
}

#[test] 
fn minimal_case() {
    let tokens = vec![
        Token::Config(String::from("locales a")),
        Token::Key(String::from("key")),
        Token::Locale(String::from("a")),
        Token::Value(String::from("value")),
    ];

    let keys = parse_tokens(tokens)
    .expect("should be ok");

    assert_eq!(
        keys,
        vec![
            Key { 
                id: String::from("key"), 
                comment: None, 
                entries: vec![String::from("value")] 
            }
        ]
    )
}

#[test] 
fn key_comment() {
    let tokens = vec![
        Token::Config(String::from("locales a")),
        Token::Comment(String::from("hi!")),
        Token::Key(String::from("key")),
        Token::Locale(String::from("a")),
        Token::Value(String::from("value")),
    ];

    let keys = parse_tokens(tokens)
    .expect("should be ok");

    assert_eq!(
        keys,
        vec![
            Key { 
                id: String::from("key"), 
                comment: Some(String::from("hi!")), 
                entries: vec![String::from("value")] 
            }
        ]
    )
}

#[test] 
fn entry_comments() {
    let tokens = vec![
        Token::Config(String::from("locales a")),
        Token::Key(String::from("key")),
        Token::Comment(String::from("hi!")),
        Token::Locale(String::from("a")),
        Token::Value(String::from("value")),
    ];

    let keys_1 = parse_tokens(tokens)
    .expect("should be ok");

    let tokens = vec![
        Token::Config(String::from("locales a")),
        Token::Key(String::from("key")),
        Token::Locale(String::from("a")),
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
                id: String::from("key"), 
                comment: Some(String::from(" # Locale notes\n- *a*: hi!\n")), 
                entries: vec![String::from("value")] 
            }
        ]
    )
}

#[test] 
fn mutli_locales() {
    let tokens = vec![
        Token::Config(String::from("locales a b")),
        Token::Key(String::from("key")),
        Token::Locale(String::from("a")),
        Token::Value(String::from("value A")),
        Token::Locale(String::from("b")),
        Token::Value(String::from("value B")),
    ];

    let keys = parse_tokens(tokens)
    .expect("should be ok");

    assert_eq!(
        keys,
        vec![
            Key { 
                id: String::from("key"), 
                comment: None, 
                entries: vec![
                    String::from("value A"),
                    String::from("value B"), 
                ] 
            }
        ]
    )
}

#[test] 
fn missing_locales() {
    let tokens = vec![
        Token::Key(String::from("key")),
        Token::Locale(String::from("a")),
        Token::Value(String::from("value A")),
        Token::Locale(String::from("b")),
        Token::Value(String::from("value B")),
    ];

    assert!(parse_tokens(tokens).is_err());
}

#[test] 
fn missing_declared_locale() {
    let tokens = vec![
        Token::Config(String::from("locales a b")),
        Token::Key(String::from("key")),
        Token::Locale(String::from("a")),
        Token::Value(String::from("value A")),
    ];

    assert!(parse_tokens(tokens).is_err());
}

#[test] 
fn using_declared_default() {
    let tokens = vec![
        Token::Config(String::from("locales a b")),
        Token::Key(String::from("key")),
        Token::Value(String::from("value A")),
        Token::Locale(String::from("b")),
        Token::Value(String::from("value B")),
    ];

    assert!(parse_tokens(tokens).is_err());
}

#[test] 
fn using_and_not_default() {
    let tokens = vec![
        Token::Key(String::from("key")),
        Token::Locale(String::from("b")),
        Token::Value(String::from("value B")),
    ];

    assert!(parse_tokens(tokens).is_err());

    let tokens = vec![
        Token::Config(String::from("locales b")),
        Token::Key(String::from("key")),
        Token::Locale(String::from("b")),
        Token::Value(String::from("value B")),
    ];

    assert!(parse_tokens(tokens).is_ok());
}
