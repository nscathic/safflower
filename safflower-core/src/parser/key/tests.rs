#![allow(clippy::unwrap_used)]

use super::*;

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
