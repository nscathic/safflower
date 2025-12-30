use super::*;

#[test]
fn bad_names() {
    for n in [
        "",
        "0",
        "   ",
        "A B",
        "_a_",
        "$8",
        "44044",
    ] {
        assert!(Name::try_from(n).is_err(), "src: \"{n}\"");
    }
}

#[test]
fn ok_names() {
    for n in [
        "a",
        "AB",
        "num0",
        "a_small",
        "f0-0",
    ] {
        assert!(Name::try_from(n).is_ok(), "src: \"{n}\"");
    }
}

