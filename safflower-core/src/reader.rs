use std::{fs::File, io::{BufRead, BufReader, Read}, path::{Path, PathBuf}};

use crate::name::{Name, NameBuilder, ValidChar};

type Result<T> = std::result::Result<T, ReadError>;

mod error;
pub use error::ReadError;

#[cfg(test)]
mod tests;

#[must_use]
pub struct Reader<R: Read> {
    path: PathBuf,
    buf: BufReader<R>,   
}
impl<R: Read> Reader<R> {
    /// Reads until a valid char is gotten
    fn read_char(&mut self) -> Result<Option<char>> {
        let mut buf = [b' '; 1];
        let mut char_u32 = 0u32;

        for _ in 0..4 {
            match self.buf.read(&mut buf) {
                Ok(0) => return Ok(None),
                Err(e) => return Err(ReadError::Io(self.path.clone(), e)),
                Ok(_) => {},
            }

            char_u32 = 
                (char_u32 << 8) 
                | u32::from(unsafe { *buf.get_unchecked(0) });
            

            if let Some(c) = char::from_u32(char_u32) {
                return Ok(Some(c));
            }
        }

        Err(ReadError::NotUtf8)
    }

    fn read_line(&mut self) -> Result<String> {
        let mut buf = String::new();
        self.buf.read_line(&mut buf)
        .map_err(|e| ReadError::Io(self.path.clone(), e))?;

        // Remove comment part, if any
        if let Some(i) = buf.find('#') {
            buf = unsafe { buf.get_unchecked(..i).to_string() };
        }

        Ok(buf)
    }
    
    #[allow(clippy::map_unwrap_or)]
    fn read_config(&mut self) -> Result<Token> {
        let buf = self.read_line()?;

        let (key, args) = buf
        .split_once(char::is_whitespace)
        .map(|(k, a)| (k.to_string(), a.trim().to_string()))
        .unwrap_or_else(|| (buf, String::new()));

        Ok(Token::Config(key, args))
    }
    
    fn read_comment(&mut self) -> Result<Token> {
        // Everything after the # is a comment
        let mut buf = String::new();
        self.buf.read_line(&mut buf)
        .map_err(|e| ReadError::Io(self.path.clone(), e))?;

        while buf.ends_with(char::is_whitespace) { _ = buf.pop(); }
        Ok(Token::Comment(buf))
    }
    
    #[allow(clippy::arithmetic_side_effects)]
    fn read_value(&mut self) -> Result<Token> {
        let mut buf = Vec::new();
        loop {
            self.buf.read_until(b'"', &mut buf)
            .map_err(|e| ReadError::Io(self.path.clone(), e))?;
            
            // We catch escaped quotes here
            if !buf.ends_with(b"\\\"") { 
                // Buffer should end with the quote we use as delimiter
                if buf.pop() != Some(b'"') {
                    return Err(ReadError::UnmatchedQuote);
                }
                break; 
            }

            // Buffer ends with \", but we want to remove the escaping slash
            _ = buf.swap_remove(buf.len() - 2);
        }

        let value = String::from_utf8(buf)
        .map_err(|_| ReadError::NotUtf8)?;
        
        Ok(Token::Value(value))
    }
    
    fn read_name(&mut self, first: char) -> Result<Token> {
        /* We know we are reading a name, so it must be either a key or a locale
           Since the structure is not completely unambiguous, we can't be sure 
           which it is until we see the next thing...
           E.g.

           key: loc "val"

           key: l1 "val" l2 "val"

           Buuut we could just have a mandatory no-space colon after the key.
        */

        let mut name = NameBuilder::new(first)?;
        
        while let Some(c) = self.read_char()? {
            match c {
                c if c.is_whitespace() => break,

                ':' => return Ok(Token::Key(name.build())),

                c => name.add(c)?,
            }
        }

        Ok(Token::Locale(name.build()))
    }
}
impl Reader<&'static [u8]> {
    pub fn from_bytes(source: &'static [u8]) -> Self {
        Self {
            path: PathBuf::new(),
            buf: BufReader::new(source),
        }
    }
}
impl Reader<File> {
    /// Constructs a new reader of the file at `path`.
    /// 
    /// # Errors
    /// Forwarded from `File::open`.
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        File::open(&path)
        .map(BufReader::new)
        .map(|buf| Self { 
            path: path.as_ref().into(),
            buf ,
        })
    }
}
impl<R: Read> Iterator for Reader<R> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        let first = loop {
            match self.read_char() {
                Ok(Some(c)) if !c.is_whitespace() => break c,
                Ok(Some(_)) => {},
                Ok(None) => return None,
                Err(e) => return Some(Err(e)),
            }
        };

        let out = match first {
            '!' => self.read_config(),
            '#' => self.read_comment(),
            '"' => self.read_value(),
            c if ValidChar::is_valid(c) => self.read_name(first),
            
            c => Err(ReadError::InvalidChar(c)),
        };
        
        Some(out)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    /// Line started with !, followed by a key and maybe args
    Config(String, String),
    /// Line started with # ended by newline
    Comment(String),
    
    /// The key part of an entry
    Key(Name),
    /// The locale part of an entry
    Locale(Name),
    /// The value part of an entry, surrounded by "
    Value(String),
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(k, a) => write!(f, "Config({k}: {a})"),
            Self::Comment(c) => write!(f, "Comment({c})"),
            Self::Key(name) => write!(f, "Key({name})"),
            Self::Locale(name) => write!(f, "Locale({name})"),
            Self::Value(v) => write!(f, "Value({v})"),
        }
    }
}
