use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("IO failure for \"{0}\": {1}")]
    Io(PathBuf, std::io::Error),
    #[error("could not read valid UTF8")]
    NotUtf8,

    #[error("key or locale cannot contain '{0}'")]
    NameInvalid(char),
    #[error("key or locale cannot start with '{0}'")]
    NameInvalidFirst(char),
    #[error("unexpected EOF")]
    EOF,
    #[error("unexpected char '{0}'")]
    InvalidChar(char),
    #[error("unexpected EOF before terminating quote")]
    UnmatchedQuote,
    #[error("name cannot be empty")]
    EmptyName,
}
