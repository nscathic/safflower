use thiserror::Error;

use crate::{parser::ParseError, reader::ReadError};

#[derive(Debug, Error)]
pub enum Error {
    #[error("reading error: {0}")]
    Read(ReadError),
    #[error("parsing error: {0}")]
    Parse(ParseError),
}
impl From<ReadError> for Error {
    fn from(value: ReadError) -> Self {
        Self::Read(value)
    }
}
impl From<ParseError> for Error {
    fn from(value: ParseError) -> Self {
        Self::Parse(value)
    }
}
