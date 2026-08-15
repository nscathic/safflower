#![doc = include_str!("../readme.md")]
use unicode_segmentation::UnicodeSegmentation;

pub mod error;
pub mod name;
pub mod reader;
pub mod parser;
pub mod generator;

fn shorten(line: impl AsRef<str>) -> String {
    let graphemes = line
    .as_ref()
    .graphemes(true)
    .take(33)
    .collect::<Vec<_>>();
    
    if graphemes.len() <= 32 {
        return graphemes
        .into_iter()
        .collect();
    }

    graphemes
    .into_iter()
    .take(29)
    .chain([".", ".", "."])
    .collect()
}
