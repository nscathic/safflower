use std::path::PathBuf;
use thiserror::Error;

use crate::{parser::ParseError, reader::ReadError};

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error for file \"{0}\": {1}")]
    Io(PathBuf, std::io::Error),
    #[error("reading error: {0}")]
    Read(ReadError),
    #[error("parsing error in file \"{0}\": {1}")]
    Parse(PathBuf, ParseError),
}
impl From<ReadError> for Error {
    fn from(value: ReadError) -> Self {
        Self::Read(value)
    }
}
