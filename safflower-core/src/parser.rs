use crate::{
    error::Error, name::Name, reader::{ReadError, Token}, shorten, validate_char
};

mod error;
mod config;
pub use error::ParseError;
use config::Configuration;

#[cfg(test)]
mod tests;

#[derive(Default)]
/// Parses iterators of safflower tokens.
pub struct Parser {
    config: Configuration,
    keys: Vec<TempKey>,

    working_comment: Option<String>,
    working_key: Option<TempKey>,
}
impl Parser {
    /// Parses a token iterator.
    /// 
    /// # Errors 
    /// If something is unparsable.
    pub fn parse(
        &mut self, 
        tokens: impl Iterator<Item = Result<Token, ReadError>>,
    ) -> Result<(), Error> {
        let mut locale = None;

        let mut locale_style = None;

        for token in tokens { match token? {
            Token::Config(c) => {
                self.config.parse_config(&c)?;

                // Check if it was locales
                if c.starts_with("locales") {
                    locale_style = Some(self.config.locales.len());
                }
            },

            // This will get put on whatever is next
            Token::Comment(c) => self.working_comment = Some(c),

            Token::Key(k) => self.parse_key(locale_style, k)?,
            
            Token::Locale(l) => self.parse_locale(&mut locale, l)?,

            Token::Value(v) => self.parse_value(
                locale, 
                locale_style, 
                v,
            )?,
        }}

        // Push the last dangling key
        if let Some(key) = self.working_key.take() { self.add_key(key)?; }
        
        Ok(())
    }

    fn parse_locale(
        &self, 
        locale: &mut Option<usize>, 
        id: Name,
    ) -> Result<(), ParseError> {
        if self.working_key.is_none() { return Err(ParseError::LocaleNoKey); }
        
        *locale = Some(
            self.config
            .find_locale(&id)
            .ok_or_else(|| ParseError::UndeclaredLocale(id.into()))?
        );

        Ok(())
    }
    
    fn parse_key(
        &mut self, 
        locale_style: Option<usize>, 
        id: Name,
    ) -> Result<(), ParseError> {
        if let Some(old_key) = self.working_key.take() { 
            self.add_key(old_key)?; 
        }
        let locale_count = locale_style.map_or(1, |c| c);
        
        let comment = self.working_comment.take();

        self.working_key = Some(TempKey {
            id,
            comment,
            entries: vec![None; locale_count],
        });
        Ok(())
    }

    fn add_key(&mut self, key: TempKey) -> Result<(), ParseError> {
        if self.keys.iter().any(|k| k.id == key.id) {
            return Err(ParseError::DuplicateKey(key.id.into()));
        }
        self.keys.push(key);
        Ok(())
    }

    fn parse_value(
        &mut self,
        locale: Option<usize>, 
        locale_style: Option<usize>, 
        v: String,
    ) -> Result<(), ParseError> {
        let Some(key) = &mut self.working_key else { 
            return Err(ParseError::ValueNoKey) 
        };

        let index = match locale_style {
            None => 0,
            Some(_) => locale.ok_or_else(|| 
                ParseError::UsingDefaultLocale(
                    key.id.to_str().to_string(),
                    shorten(&v),
                )
            )?,
        };

        if key.entries[index].is_some() {
            let locale = self.config.locales[index].clone();
            return Err(ParseError::DuplicateEntry(
                locale.into(),
                key.id.to_str().to_string(),
            ));
        }

        let comment = self.working_comment.take();

        key.entries[index] = Some(Entry {
            value: v,
            comment,
        });
        
        Ok(())
    }
    
    /// Collects all the parsed data.
    /// 
    /// # Errors
    /// If any key fails to be validated.
    pub fn collect(self) -> Result<ParsedData, ParseError> {
        let Self {
            config: head,
            keys,
            ..
        } = self;

        let keys = keys
        .into_iter()
        .map(|key| key.validate(&head.locales))
        .collect::<Result<_, ParseError>>()?;

        let locales = head.locales;

        Ok(ParsedData {
            locales,
            keys,
        })
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
                validate_char(c)
                .map_err(|c| ParseError::ArgBadChar(
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
    .map(|(i, e)| e.ok_or_else(|| ParseError::MissingLocale(
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
