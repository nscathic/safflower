#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::println;

use super::*;

fn read(source: &'static str) -> Result<Vec<Token>> {
    Reader::<&[u8]>::from_bytes(source.as_bytes()).collect()
}

#[test]
fn empty() {
    for source in [
        "",
        " ",
        "\n\n",
        " \t \n ",
    ] {
        let mut reader = Reader::<&[u8]>::from_bytes(source.as_bytes());
        assert!(reader.next().is_none(), "src: '{source}'");
    }
}

#[test]
fn key() {
    let correct = vec![Token::Key(Name::try_from("key").unwrap())];
    for source in [
        "key:",
        "  key: ",
        "\nkey:",
        " \t \n key: ",
    ] {
        let tokens = read(source).expect(source);
        assert_eq!(tokens, correct, "src: '{source}'");
    }
}

#[test]
fn loc() {
    let correct = vec![
        Token::Locale(Name::try_from("loc").unwrap()),
        Token::Value(String::new()),
    ];
    for source in [
        "loc \"\"",
        "  loc \"\" ",
        "\nloc\n\"\"",
        " \t \n loc  \t\"\" ",
    ] {
        let tokens = read(source).expect(source);
        assert_eq!(tokens, correct, "src: '{source}'");
    }
}

#[test]
fn comment() {
    for (source, comment) in [
        ("#\n", ""),
        ("# \n", ""),
        ("\n#\n\n", ""),
        (" #\t \n ", ""),
        ("#text\n", "text"),
        ("#text and spaaace    \n", "text and spaaace"),
        ("#  indent\n", "  indent"),
        ("#one two\n", "one two"),
    ] {
        println!("{source}");
        let tokens = read(source).expect(source);
        assert_eq!(
            tokens, 
            vec![Token::Comment(comment.to_string())], 
            "src: '{source}'",
        );
    }
}

#[test]
fn comment_config() {
    let source = "!locales en #comment";
    let tokens = read(source).expect(source);
    assert_eq!(
        tokens,
        vec![Token::Config(
            String::from("locales"),
            String::from("en")
        )]
    );
}

#[test]
fn comment_others() {
    let key = Token::Key(Name::try_from("key").unwrap());
    let loc = Token::Locale(Name::try_from("loc").unwrap());
    let val = Token::Value(String::from("value"));
    let com = Token::Comment(String::from("comment"));
    
    for (source, tokens) in [
        ("key: #comment\n loc \"value\"", [&key, &com, &loc, &val]),
        ("key: loc \"value\" #comment",   [&key, &loc, &val, &com]),
    ] {
        let expected = read(source).expect(source);
        assert_eq!(
            expected,
            tokens.map(std::clone::Clone::clone),
            "source: {source}"
        );
    }
}

#[test]
fn config() {
    for (source, expected) in [
        ("!\n", Token::Config(String::new(), String::new())),
        ("! \n", Token::Config(String::new(), String::new())),
        ("\n!\n\n", Token::Config(String::new(), String::new())),
        (" !\t \n ", Token::Config(String::new(), String::new())),
        ("!text\n", Token::Config("text".into(), String::new())),
        ("!one two\n", Token::Config("one".into(), "two".into())),
    ] {
        let tokens = read(source).expect(source);
        assert_eq!(
            tokens, 
            vec![expected], 
            "src: '{source}'"
        );
    }
}

#[test]
fn value() {
    let correct = vec![Token::Value(String::from("key"))];
    for source in [
        "\"key\"",
        "  \"key\"  ",
        "\n\"key\"\n",
        " \t \n \"key\"  \t",
    ] {
        let tokens = read(source).expect(source);
        assert_eq!(tokens, correct, "src: '{source}'");
    }
}

#[test]
fn one_line() {
    for source in [
        "key: loc \"value\"",
        "key:loc \"value\"",
        "key:\n loc \"value\"",
        "key: loc\n \"value\"",
        "key: loc \"value\"\n",
        "key:  loc \"value\"   ",
    ] {
        let tokens = read(source).expect(source);
        assert_eq!(tokens, vec![
            Token::Key(Name::try_from("key").expect(source)),
            Token::Locale(Name::try_from("loc").expect(source)),
            Token::Value(String::from("value")),
        ], "src: '{source}'");
    }
}

#[test]
fn multiline() {
    let source = "
    #this is a comment
    key:
        en \"english\"
        it \"italiano\"
        # another comment
        sv \"svenska\"
    ";

    let tokens = read(source).expect(source);
    assert_eq!(tokens, vec![
        Token::Comment(String::from("this is a comment")),
        Token::Key(Name::try_from("key").expect(source)),
        Token::Locale(Name::try_from("en").expect(source)),
        Token::Value(String::from("english")),
        Token::Locale(Name::try_from("it").expect(source)),
        Token::Value(String::from("italiano")),
        Token::Comment(String::from(" another comment")),
        Token::Locale(Name::try_from("sv").expect(source)),
        Token::Value(String::from("svenska")),
    ]);
}

#[test]
fn enquoted() {
    let source = "
    key:
        en \"I speak \\\"english\\\"\"
    ";

    let tokens = read(source).expect(source);
    assert_eq!(tokens, vec![
        Token::Key(Name::try_from("key").expect(source)),
        Token::Locale(Name::try_from("en").expect(source)),
        Token::Value(String::from("I speak \"english\"")),
    ]);
}
