use super::*;

fn read_all(source: &str) -> Vec<Token> {
    CharReader::new(source).collect()
}

#[test]
fn empty() {
    for source in [
        "",
        " ",
        "\n\n",
        " \t \n ",
    ] {
        let mut reader = CharReader::new(source);
        assert_eq!(reader.next(), None, "src: '{source}'");
    }
}

#[test]
fn key() {
    let correct = vec![Token::Key(String::from("key"))];
    for source in [
        "key:",
        "  key : ",
        "\nkey\n:",
        " \t \n key  \t: ",
    ] {
        let tokens = read_all(source);
        assert_eq!(tokens, correct, "src: '{source}'");
    }
}

#[test]
fn loc() {
    let correct = vec![
        Token::Locale(String::from("loc")),
        Token::Value(String::new()),
    ];
    for source in [
        "loc \"\"",
        "  loc \"\" ",
        "\nloc\n\"\"",
        " \t \n loc  \t\"\" ",
    ] {
        let tokens = read_all(source);
        assert_eq!(tokens, correct, "src: '{source}'");
    }
}

#[test]
fn comment() {
    for (source, comment) in [
        ("#\n", ""),
        ("# \n", " "),
        ("\n#\n\n", ""),
        (" #\t \n ", "\t "),
        ("#text\n", "text"),
        ("#one two\n", "one two"),
    ] {
        let tokens = read_all(source);
        assert_eq!(tokens, vec![
            Token::Comment(comment.to_string()), 
        ], "src: '{source}'");
    }
}

#[test]
fn comment_config() {
    let source = "!locales en #comment";
    let tokens = read_all(source);
    assert_eq!(
        tokens,
        vec![Token::Config(String::from("locales en "))]
    );
}

#[test]
fn comment_others() {
    let key = Token::Key(String::from("key"));
    let loc = Token::Locale(String::from("loc"));
    let val = Token::Value(String::from("value"));
    let com = Token::Comment(String::from("comment"));
    
    for (source, tokens) in [
        ("key: #comment\n loc \"value\"", [&key, &com, &loc, &val]),
        ("key: loc \"value\" #comment",   [&key, &loc, &val, &com]),
    ] {
        let expected = read_all(source);
        assert_eq!(
            expected,
            tokens.map(|t| t.clone()),
            "source: {source}"
        );
    }
}

#[test]
fn config() {
    for (source, comment) in [
        ("!\n", ""),
        ("! \n", " "),
        ("\n!\n\n", ""),
        (" !\t \n ", "\t "),
        ("!text\n", "text"),
        ("!one two\n", "one two"),
    ] {
        let tokens = read_all(source);
        assert_eq!(
            tokens, 
            vec![Token::Config(comment.to_string())], 
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
        let tokens = read_all(source);
        assert_eq!(tokens, correct, "src: '{source}'");
    }
}

#[test]
fn one_line() {
    for source in [
        "key: loc \"value\"",
        "key:loc\"value\"",
        "key:\n loc \"value\"",
        "key: loc\n \"value\"",
        "key: loc \"value\"\n",
        "key   :  loc \"value\"   ",
    ] {
        let tokens = read_all(source);
        assert_eq!(tokens, vec![
            Token::Key(String::from("key")),
            Token::Locale(String::from("loc")),
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

    let tokens = read_all(source);
    assert_eq!(tokens, vec![
        Token::Comment(String::from("this is a comment")),
        Token::Key(String::from("key")),
        Token::Locale(String::from("en")),
        Token::Value(String::from("english")),
        Token::Locale(String::from("it")),
        Token::Value(String::from("italiano")),
        Token::Comment(String::from(" another comment")),
        Token::Locale(String::from("sv")),
        Token::Value(String::from("svenska")),
    ])
}
