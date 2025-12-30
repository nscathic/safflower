use thiserror::Error;

use crate::reader::Token;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("empty config key")]
    ConfigEmptyKey,
    #[error("unrecognised key \"{0}\"")]
    ConfigUnknownKey(String),
    #[error("missing values for config \"{0}\"")]
    ConfigMissingValues(&'static str),
    
    #[error("duplicate locale \"{0}\"")]
    DuplicateLocale(String),
    #[error("duplicate entry for locale \"{0}\" in key \"{1}\"")]
    DuplicateEntry(String, String),
    #[error("encountered locale \"{0}\", but it has not been declared")]
    UndeclaredLocale(String),
    #[error("entry \"{0}\" is missing locale [{1}]")]
    EntryMissingLocale(String, String),

    #[error("there are no locales set up; please do so with \"!locales \
        LOCALES+\"")]
    NoLocales,

    #[error("value contains nested or an unclosed opening brace '{{'")]
    NestedBrace,
    #[error("value contains unopened closing brace '}}'")]
    ExtraClosingBrace,

    #[error("line \"{0}\" contains argument \"{1}\" with invalid char \
        \"{2}\", but must be only alphanumeric, '-', or '_'")]
    ArgBadChar(String, String, char),
    #[error("line \"{0}\" contains argument \"{1}\" that starts with \
        \"{2}\", but it must start with an alphabetic character or be all \
        digits")]
    ArgBadStart(String, String, char),

    #[error("entry \"{1}\" for key \"{0}\" has arguments {2:?}, which \
        does not match {3:?} from the key's first entry")]
    ArgumentMismatch(String, String, Vec<String>, Vec<String>),
    #[error("line may not start with {0}")]
    UnexpectedToken(Token),
    #[error("expected locale to follow, but token stream ended")]
    ExpectedLocale,
    #[error("expected value to follow, but token stream ended")]
    ExpectedValue,
}
