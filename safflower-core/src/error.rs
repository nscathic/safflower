use std::path::PathBuf;
use thiserror::Error;

use crate::{parser::ParseError, reader::ReadError};

#[must_use]
#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error for file \"{0}\": {1}")]
    Io(PathBuf, std::io::Error),
    #[error("reading error in file \"{0}\": {1}")]
    Read(PathBuf, ReadError),
    #[error("parsing error in file \"{0}\": {1}")]
    Parse(PathBuf, ParseError),
}
