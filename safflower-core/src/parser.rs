use crate::{
    error::Error, name::Name, reader::{ReadError, Token}, shorten
};

mod error;
mod config;
pub use error::ParseError;
use config::Configuration;

#[cfg(test)]
mod tests;

/// Parses iterators of safflower tokens.
pub struct Parser<I: Iterator<Item = Result<Token, ReadError>>> {
    tokens: I,
    buffer: Option<Token>,

    config: Configuration,
    keys: Vec<TempKey>,

    comment: Option<String>,
}
impl<I: Iterator<Item = Result<Token, ReadError>>> Parser<I> {
    pub fn new(tokens: I) -> Self {
        Self {
            tokens,
            buffer: None,

            config: Configuration::default(),
            keys: vec![],
            
            comment: None,
        }
    }

    /// Parses all tokens and returns the parsed data.
    /// 
    /// # Errors 
    /// If something is unparsable.
    pub fn parse(mut self) -> Result<ParsedData, Error> {
        loop {
            let token = match self.buffer.take() {
                Some(t) => Some(t),
                None => self.tokens.next().transpose()?,
            };
            
            match token {
                Some(t) => self.parse_token(t)?,
                None => break,
            }
        }

        let Self {
            config,
            keys, 
            ..
        } = self;

        let keys = keys
        .into_iter()
        .map(|key| key.validate(&config.locales))
        .collect::<Result<_, ParseError>>()?;

        let locales = config.locales;

        Ok(ParsedData {
            locales,
            keys,
        })
    }

    fn parse_token(&mut self, token: Token) -> Result<(), Error> {
        // The are only a few valid token sequences:
        // 1) !config values
        // 2) key: (locale "value")+
        // 3) #comment (2)
        match token {
            Token::Config(c) => {
                self.config.parse_config(&c)?;
                // In case a comment was read before, it should be removed
                self.comment = None;
            },

            // Buffer a comment
            Token::Comment(c) => self.comment = Some(c),

            // Read a key (and the following locales and values)
            Token::Key(id) => self.parse_key(id)?,

            // We can't start a line with a locale or value
            t => Err(ParseError::UnexpectedToken(t))?,
        }

        Ok(())
    }

    fn parse_key(&mut self, id: Name) -> Result<(), Error> {
        // We have a key, so we must now get all the locale-value pairs
        let mut entries = vec![None; self.config.locale_count()];
        let mut did_something = false;

        let comment = self.comment.take();
        loop {
            let Some(locale) = self.get_locale()? else { break; };
            let index = self.config
            .find_locale(&locale)
            .ok_or_else(|| ParseError::UndeclaredLocale(locale.into()))?;

            let Some(value) = self.get_value()? else { break; };
            let comment = self.comment.take();

            entries[index] = Some(Entry { value, comment });
            did_something = true;
        }

        if !did_something {
            return Err(ParseError::ExpectedLocale.into());
        }

        let key = TempKey {
            id,
            comment,
            entries,
        };

        self.add_key(key).map_err(Into::into)
    }

    fn get_locale(&mut self) -> Result<Option<Name>, Error> {
        for t in self.tokens.by_ref() {
            match t? {
                Token::Comment(c) => self.comment = Some(c),
                Token::Locale(id) => return Ok(Some(id)),

                // We expect key - loc - val - loc - val ...
                // until there is a key again
                Token::Key(id) => { 
                    self.buffer = Some(Token::Key(id)); 
                    return Ok(None);
                }

                t => Err(ParseError::UnexpectedToken(t))?,
            }
        }
        Ok(None)
    }

    fn get_value(&mut self) -> Result<Option<String>, Error> {
        for t in self.tokens.by_ref() {
            match t? {
                Token::Comment(c) => self.comment = Some(c),
                Token::Value(value) => return Ok(Some(value)),

                t => Err(ParseError::UnexpectedToken(t))?,
            }
        }
        Err(ParseError::ExpectedValue)?
    }

    fn add_key(&mut self, key: TempKey) -> Result<(), ParseError> {
        // Check if an old key matches the new one
        if let Some(old_key) = self.keys.iter_mut().find(|k| k.id == key.id) {
            let TempKey { id, comment, entries } = key;

            // If no entries overlap, it's ok, otherwise it's an error
            for (i, e) in entries.into_iter().enumerate() {
                if e.is_none() { continue; }

                if old_key.entries[i].is_some() {
                    return Err(ParseError::DuplicateEntry(
                        id.into(),
                        self.config.locales[i].to_str().into(),
                    ));
                }

                old_key.entries[i] = e;
            }

            // Join the comments as well
            old_key.comment = match (old_key.comment.take(), comment) {
                (None, None) => None,
                (None, Some(c)) | (Some(c), None) => Some(c),
                (Some(c1), Some(c2)) => Some(c1 + &c2),
            };

            return Ok(());
        }

        self.keys.push(key);

        Ok(())
    }
}

/// The collected data once the parsing is finished.
pub struct ParsedData {
    pub locales: Vec<Name>,
    pub keys: Vec<Key>,
}

#[derive(Debug, PartialEq, Eq)]
struct TempKey {
    id: Name,
    comment: Option<String>,
    entries: Vec<Option<Entry>>,
}
impl TempKey {
    fn validate(self, locales: &[Name]) -> Result<Key, ParseError> {
        if locales.is_empty() { return Err(ParseError::NoLocales); }
        
        let Self { id, comment, entries } = self;

        let (entries, comments) = get_entries(entries, &id, locales)?;
        let comment = get_comment(comments, comment, locales);
        let arguments = get_arguments(&entries, &id, locales)?;

        Ok(Key {
            id,
            arguments,
            comment,
            entries,
        })
    }
}

fn get_arguments(
    entries: &[String], 
    id: &Name,
    locales: &[Name],
) -> Result<Vec<String>, ParseError> {
    let arguments = extract_arguments(&entries[0])?;
        
    let mismatch = entries
    .iter()
    .enumerate()
    .skip(1)
    .map(|(i, e)| (i, extract_arguments(e)))
    .find(|(_, a)| !a.as_ref().is_ok_and(|a| a == &arguments));

    if let Some((index, result)) = mismatch {
        let args = result?;
        return Err(ParseError::ArgumentMismatch(
            id.to_str().to_string(), 
            locales[index].to_str().to_string(),
            args,
            arguments,
        ));
    }

    Ok(arguments)
}

fn extract_arguments(key: &str) -> Result<Vec<String>, ParseError> {
    let mut arguments = Vec::new();
    let mut argument = String::new();
    let mut opened = false;
    let mut unnamed_indexer = 0;
    let mut formatting = false;

    for c in key.chars() {
        match c {
            '{' if opened => return Err(ParseError::NestedBrace),
            '{' => { opened = true; },

            '}' if !opened => return Err(ParseError::ExtraClosingBrace),
            '}' => {
                if argument.is_empty() {
                    argument = format!("{unnamed_indexer}");
                    unnamed_indexer += 1;
                }
                else if !argument.starts_with(
                    |c: char| c.is_ascii_alphabetic()
                ) && !argument.chars().all(char::is_numeric)  {
                    return Err(ParseError::ArgBadStart(
                        key.to_string(), 
                        shorten(&argument), 
                        c,
                    ))
                }

                if !arguments.contains(&argument) {                        
                    arguments.push(argument);
                }

                argument = String::new();
                opened = false;
                formatting = false;
            }

            ':' if opened => formatting = true,

            // Don't copy the formatting part
            c if opened && !formatting => argument.push(
                Name::validate_char(c)
                .map_err(|_| ParseError::ArgBadChar(
                    shorten(key), 
                    shorten(&argument),
                    c,
                ))?
            ),
            
            _ => (),
        }
    }

    Ok(arguments) 
}

fn get_comment(
    comments: Vec<Option<String>>,
    key_comment: Option<String>,
    locales: &[Name],
) -> Option<String> {
    let locale_comment = comments
    .into_iter()
    .enumerate()
    .filter_map(|(i, comment)| 
        comment.map(|c| format!("- *{}*: {c}\n", locales[i].to_str()))
    )
    .collect::<String>();

    if locale_comment.is_empty() { return key_comment; }
    
    Some(format!(
        "{} # Locale notes\n{locale_comment}", 
        key_comment.unwrap_or_default(),
    ))
}

fn get_entries(
    entries: Vec<Option<Entry>>,
    id: &Name,
    locales: &[Name],
) -> Result<(Vec<String>, Vec<Option<String>>), ParseError> {
    entries
    .into_iter()
    .enumerate()
    .map(|(i, e)| e.ok_or_else(|| ParseError::EntryMissingLocale(
        shorten(id),
        locales[i].to_str().to_string()
    )))
    .collect::<Result<Vec<Entry>,_>>()
    .map(|ok| ok
        .into_iter()
        .map(|e| (e.value, e.comment))
        .unzip()
    )
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Key {
    pub id: Name,
    pub arguments: Vec<String>,
    pub comment: Option<String>,
    pub entries: Vec<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Entry {
    pub value: String,
    pub comment: Option<String>,
}
