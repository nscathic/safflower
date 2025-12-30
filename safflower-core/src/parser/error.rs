use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(std::io::Error),
    #[error("empty config key")]
    EmptyKey,
    #[error("unrecognised key \"{0}\"")]
    UnknownKey(String),
    
    #[error("duplicate locale \"{0}\"")]
    DuplicateLocale(String),
    #[error("duplicate key \"{0}\"")]
    DuplicateKey(String),
    #[error("duplicate entry for locale \"{0}\" in key \"{1}\"")]
    DuplicateEntry(String, String),
    #[error("missing values for config \"{0}\"")]
    MissingValues(&'static str),
    #[error("missing value for key \"{0}\"")]
    KeyNoValue(String),
    #[error("value was started, but no ending quote was found")]
    UnfinishedQuote,
    #[error("encountered a locale but no key is found preceeding it")]
    LocaleNoKey,
    #[error("encountered a value but no key is found preceeding it")]
    ValueNoKey,
    #[error("encountered locale \"{0}\", but it has not been declared")]
    UndeclaredLocale(String),
    #[error("entry \"{0}\" is missing locale [{1}]")]
    MissingLocale(String, String),
    #[error("entry \"{0}\" does not have a locale specified for line \"{1}\", \
        but you have declared locales and so must use them")]
    UsingDefaultLocale(String, String),
    #[error("locale \"{0}\" contains invalid char '{1}', but must be only \
        alphanumeric, '-', or '_'")]
    LocaleBadChar(String, char),
    #[error("locale \"{0}\" does not start with alphabetical char")]
    LocaleBadStart(String),

    #[error("there are no locales set up; please do so with \"!locales \
        [LOCALES]\"")]
    NoLocales,
    #[error("key \"{0}\" contains invalid char \"{1}\", but must be only \
        alphanumeric, '-', or '_'")]
    KeyBadChar(String, char),
    #[error("key \"{0}\" does not start with alphabetical char")]
    KeyBadStart(String),

    #[error("value contains nested or an unclosed opening brace '{{'")]
    NestedBrace,
    #[error("value contains unopened closing brace '}}'")]
    ExtraClosingBrace,

    #[error("line \"{0}\" contains argument \"{1}\" with invalid char \
        \"{2}\", but must be only alphanumeric, '-', or '_'")]
    ArgBadChar(String, String, char),
    #[error("line \"{0}\" contains argument \"{1}\" that starts with \
        \"{2}\", but it must start with an alphabetic character")]
    ArgBadStart(String, String, char),

    #[error("entry \"{1}\" for key \"{0}\" has arguments {2:?}, which \
        does not match {3:?} from the key's first entry")]
    ArgumentMismatch(String, String, Vec<String>, Vec<String>),
}
impl From<std::io::Error> for ParseError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
