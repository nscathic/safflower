use std::{path::{Path, PathBuf}, rc::Rc};

use crate::{
    error::Error, 
    name::Name, 
    reader::{Token, Reader, ReadError},
};

mod error;
mod config;
mod key;
mod scope;

pub use error::ParseError;
pub use key::Key;
pub use scope::Scope;
pub use config::{Configuration, Locale};
use config::Module; 
use key::KeyBuilder; 
use scope::ScopeBuilder;

#[cfg(test)]
mod tests;

/// Parses iterators of safflower tokens.
pub struct Parser {
    tokens: Box<dyn Iterator<Item = Result<Token, ReadError>>>,
    buffer: Option<Token>,

    read_paths: Vec<PathBuf>,

    config: Configuration,
    scope: ScopeBuilder,

    comment: Option<String>,
}
impl Parser {
    /// Creates a parser to read from a file path.
    /// 
    /// # Errors 
    /// If there is a problem reading the file as UTF-8.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
        let tokens = Box::new(
            Reader::new(&path)
            .map_err(|e| Error::Io(path.as_ref().into(), e))?
        );
        let read_paths = vec![path.as_ref().into()];

        Ok(Self {
            tokens,
            buffer: None,

            read_paths,

            config: Configuration::new(path.as_ref().into()),
            scope: ScopeBuilder::default(),
            
            comment: None,
        })
    }

    #[must_use]
    #[cfg(test)]
    pub fn from_text(text: &'static str) -> Self {
        let tokens = Box::new(Reader::from_bytes(text.as_bytes()));

        Self {
            tokens,
            buffer: None,

            read_paths: vec![],

            config: Configuration::new(PathBuf::from("string")),
            scope: ScopeBuilder::default(),
            
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
            scope: ScopeBuilder::default(),
            
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

        self.tokens = Box::new(
            Reader::new(&path)
            .map_err(|e| Error::Io(path.clone(), e))?
        );
        self.read_paths.push(path);

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

        let scope = self.scope.build(&self.config)?;

        let locales = self.config.into_locales();

        Ok(ParsedData {
            locales,
            scope,
        })
    }

    fn parse_token(&mut self, token: Token) -> Result<(), Error> {
        // The are only a few valid token sequences:
        // 1) !config (value)+
        // 2) key: (locale "value")+
        // 3) #comment
        
        match token {
            Token::Config(key, args) => {
                self.config.parse_config(&key, &args)?;
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
        let mut builder = KeyBuilder::new(id, self.comment.take());

        let mut did_something = false;

        loop {
            let Some(locale) = self.get_locale()? else { break; };
            let Some(value) = self.get_value()? else { break; };
            let comment = self.comment
            .take()
            .map(|c| format!("- *{}*: {c}\n", locale.key()));

            builder.add_entry(locale, value, comment);

            did_something = true;
        }

        if !did_something {
            return Err(self.config.parse_err(ParseError::ExpectedLocale));
        }

        let target_scope = self.config.get_scope();
    
        self
        .traverse_scopes(target_scope)
        .add_key(builder)
        .map_err(|e| self.config.parse_err(e))
    }

    fn get_locale(&mut self) -> Result<Option<Rc<Locale>>, Error> {
        for t in self.tokens.by_ref() {
            let token = t.map_err(|e| self.config.read_err(e))?;

            match token {
                // Token::Locale(id) => return Ok(Some(id)),
                
                Token::Locale(id) => return self.config
                    .find_locale(&id)
                    .ok_or_else(|| self.config.parse_err(
                        ParseError::UndeclaredLocale(id.to_string())
                    ))
                    .map(Some),
                
                Token::Comment(c) => self.comment = Some(c),
                
                // We expect key - loc - val - loc - val ...
                // until there is a key again (or maybe config)
                Token::Config(_, _) |
                Token::Key(_) => { 
                    self.buffer = Some(token); 
                    return Ok(None);
                }

                t @ Token::Value(_) => return Err(self.config.parse_err(
                    ParseError::UnexpectedToken(t)
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

    /// Goes through each step in the chain of scope names, finding it in the 
    /// current mods, adding it if it doesn't exist.
    fn traverse_scopes(&mut self, target_scope: Vec<Name>) -> &mut ScopeBuilder {
        let mut current = &mut self.scope;
        for part in target_scope { 
            current = current.traverse(part);
        }
        current
    }
}

#[derive(Debug)]
/// The collected data once the parsing is finished.
pub struct ParsedData {
    pub locales: Vec<Rc<Locale>>,
    pub scope: Scope,
}
