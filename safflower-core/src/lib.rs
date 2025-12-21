#![doc = include_str!("../readme.md")]

pub mod error;
pub mod reader;
pub mod parser;
pub mod generator;

const fn is_valid_char(c: char) -> bool {
    validate_char(c).is_ok()
}

const fn validate_char(c: char) -> Result<char, char> {
    match c {
        '0'..='9' | 
        'a'..='z' => Ok(c),
        
        'A'..='Z' => Ok(c.to_ascii_lowercase()),
        
        '_' |
        '-' => Ok('_'),

        c => Err(c),
    }    
}

fn shorten(line: &str) -> String {
    let mut it = line.chars();

    let mut i = 0;
    let mut cs = vec![' '; 24.min(line.len())];
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
