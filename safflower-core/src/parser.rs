use std::path::{Path, PathBuf};

use crate::{
    error::Error, 
    name::Name, 
    parser::{config::Module, scope::TempScope}, 
    reader::{CharReader, ReadError, Token},
};

mod error;
mod config;
mod key;
mod scope;
pub use error::ParseError;
pub use key::{Key, TempKey, Entry};
pub use scope::Scope;
use config::Configuration;

#[cfg(test)]
mod tests;

/// Parses iterators of safflower tokens.
pub struct Parser {
    tokens: Box<dyn Iterator<Item = Result<Token, ReadError>>>,
    buffer: Option<Token>,

    read_paths: Vec<PathBuf>,

    config: Configuration,
    keys: Vec<TempKey>,
    scope: TempScope,

    comment: Option<String>,
}
impl Parser {
    /// Creates a parser to read from a file path.
    /// 
    /// # Errors 
    /// If there is a problem reading the file as UTF-8.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
        let source = std::fs::read_to_string(&path)
        .map_err(|e| Error::Io(path.as_ref().into(), e))?;

        let tokens = Box::new(CharReader::new(&source));
        let read_paths = vec![path.as_ref().into()];

        Ok(Self {
            tokens,
            buffer: None,

            read_paths,

            config: Configuration::new(path.as_ref().into()),
            keys: vec![],
            scope: TempScope::default(),
            
            comment: None,
        })
    }

    #[must_use]
    #[cfg(test)]
    pub fn from_text(text: &str) -> Self {
        let tokens = Box::new(CharReader::new(text));

        Self {
            tokens,
            buffer: None,

            read_paths: vec![],

            config: Configuration::new(PathBuf::from("string")),
            keys: vec![],
            scope: TempScope::default(),
            
            comment: None,
        }
    }

    #[cfg(test)]
    pub fn from_vec(source: Vec<Token>) -> Self {
        let tokens = Box::new(
            source
            .into_iter()
            .map(Ok)
            .collect::<Vec<_>>()
            .into_iter()
        );

        Self {
            tokens,
            buffer: None,

            read_paths: vec![],

            config: Configuration::new(PathBuf::from("vec")),
            keys: vec![],
            scope: TempScope::default(),
            
            comment: None,
        }
    }  

    fn refill_tokens(&mut self) -> Result<bool, Error> {
        let Some(module) = self.config.pop_path() else { return Ok(false); };

        let Module { path, .. } = module;

        if self.read_paths.contains(&path) {
            return Err(self.config.parse_err(
                ParseError::ConfigDuplicateFile(path)
            ));
        }

        let source = std::fs::read_to_string(&path)
        .map_err(|e| Error::Io(
            path.clone(),
            e,
        ))?;
        
        self.read_paths.push(path);

        self.tokens = Box::new(CharReader::new(&source));

        Ok(true)
    }

    /// Parses all tokens and returns the parsed data.
    /// 
    /// # Errors 
    /// If something is unparsable.
    pub fn parse(mut self) -> Result<ParsedData, Error> {
        loop {
            let token = match self.buffer.take() {
                Some(t) => Some(t),
                None => self.tokens
                    .next()
                    .transpose()
                    .map_err(|e| self.config.read_err(e))?,
            };
            
            match token {
                Some(t) => self.parse_token(t)?,
                None => if !self.refill_tokens()? { break },
            }
        }

        // let keys = std::mem::take(&mut self.keys);

        // let keys = keys
        // .into_iter()
        // .map(|key| key.validate(&self.config.locales))
        // .collect::<Result<_, ParseError>>()
        // .map_err(|e| self.config.err(e))?;

        // let scope = Scope::create(&self.config, keys)?;
        let scope = self.scope.validate(&self.config)?;

        let locales = self.config.locales;

        Ok(ParsedData {
            locales,
            // keys,
            scope,
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
            t => return Err(self.config.parse_err(
                ParseError::UnexpectedToken(t)
            )),
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
            .ok_or_else(|| self.config.parse_err(
                ParseError::UndeclaredLocale(locale.into())
            ))?;

            let Some(value) = self.get_value()? else { break; };
            let comment = self.comment.take();

            entries[index] = Some(Entry { value, comment });
            did_something = true;
        }

        // let scope = self.config.get_scope();

        if !did_something {
            return Err(self.config.parse_err(ParseError::ExpectedLocale));
        }

        let key = TempKey {
            id,
            scope: vec![],
            comment,
            entries,
        };

        self.add_key(key).map_err(|e| self.config.parse_err(e))
    }

    fn get_locale(&mut self) -> Result<Option<Name>, Error> {
        for t in self.tokens.by_ref() {
            let token = t.map_err(|e| self.config.read_err(e))?;
            match token {
                Token::Comment(c) => self.comment = Some(c),
                Token::Locale(id) => return Ok(Some(id)),

                // We expect key - loc - val - loc - val ...
                // until there is a key again (or maybe config)
                Token::Config(_) |
                Token::Key(_) => { 
                    self.buffer = Some(token); 
                    return Ok(None);
                }

                Token::Value(v) => return Err(self.config.parse_err(
                    ParseError::UnexpectedToken(Token::Value(v))
                )),
            }
        }
        Ok(None)
    }

    fn get_value(&mut self) -> Result<Option<String>, Error> {
        for t in self.tokens.by_ref() {
            match t.map_err(|e| self.config.read_err(e))? {
                Token::Comment(c) => self.comment = Some(c),
                Token::Value(value) => return Ok(Some(value)),

                t => return Err(self.config.parse_err(
                    ParseError::UnexpectedToken(t)
                )),
            }
        }
        Err(self.config.parse_err(ParseError::ExpectedValue))
    }

    fn add_key(&mut self, key: TempKey) -> Result<(), ParseError> {
        let target_scope = self.config.get_scope();
    
        let scope = self.traverse_scopes(target_scope);

        // Check if an old key matches the new one
        if let Some(old_key) = scope.keys
        .iter_mut()
        .find(|k| k.id == key.id && k.scope == key.scope) {
            let TempKey { id, comment, entries, .. } = key;

            if old_key.entries.len() < entries.len() {
                let size_difference = entries.len() - old_key.entries.len();
                old_key.entries.append(&mut vec![None; size_difference]);
            }

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

        scope.keys.push(key);

        Ok(())
    }
    
    /// Goes through each step in the chain of scope names, finding it in the 
    /// current mods, adding it if it doesn't exist.
    fn traverse_scopes(&mut self, target_scope: Vec<Name>) -> &mut TempScope {
        let mut current = &mut self.scope;

        for part in target_scope {
            let index = current.nested
            .iter()
            .position(|(n, _)| n == &part)
            .unwrap_or_else(|| {
                let nested = TempScope::default();
                let i = current.nested.len();
                current.nested.push((part, Box::new(nested)));
                i
            });

            current = &mut current.nested[index].1;
        }

        current
    }
}

#[derive(Debug)]
/// The collected data once the parsing is finished.
pub struct ParsedData {
    pub locales: Vec<Name>,
    // pub keys: Vec<Key>,
    pub scope: Scope,
}
