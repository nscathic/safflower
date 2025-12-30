#![doc = include_str!("../readme.md")]

pub const LOCALE_FAILURE_MESSAGE: &str = "could not acquire current locale";

pub mod error;
pub mod name;
pub mod reader;
pub mod parser;
pub mod generator;

fn shorten(line: impl AsRef<str>) -> String {
    let len = line.as_ref().len();
    let mut it = line.as_ref().chars();

    let mut i = 0;
    let mut cs = vec![' '; 24.min(len)];
    for c in it.by_ref() {
        cs[i] = c;
        i += 1;
        if i == 24 { break; }
    }

    if it.next().is_some() {
        for c in cs.iter_mut().skip(21) { *c = '.'; }
    }

    cs.iter().collect()
}
