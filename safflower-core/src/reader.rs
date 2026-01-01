use crate::name::Name;

mod error;
pub use error::ReadError;

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct CharReader {
    chars: Vec<char>,
    buffer: Option<char>,
}
impl CharReader {
    #[must_use]
    pub fn new(source: &str) -> Self {
        Self {
            chars: source.chars().rev().collect(),
            buffer: None,
        }
    }

    fn read_comment(&mut self) -> Token {
        let mut comment = String::new();

        while let Some(c) = self.chars.pop() {
            if c == '\n' { break; }
            comment += &c.to_string();
        }

        Token::Comment(comment)
    }
    
    fn read_config(&mut self) -> Token {
        let mut line = String::new();
        let mut add = true;

        while let Some(c) = self.chars.pop() {
            if c == '\n' { break; }
            if c == '#' { add = false; }
 
            if add { line += &c.to_string(); }
        }
        Token::Config(line)
    }
    
    fn read_value(&mut self) -> Result<Token, ReadError> {
        let mut value = String::new();
        loop {
            match self.chars.pop() {
                Some('"') => if value.ends_with('\\') {
                    // Quote may be escaped...
                    _ = value.pop();
                    value += "\"";
                } 
                else {
                    // ..or end the value
                    return Ok(Token::Value(value))
                },

                // Any random char is added to the buffer, unchecked.
                Some(c) => value += &c.to_string(),

                // If the iterator finished without closing the quote, you have
                // some problems in your file.
                None => return Err(ReadError::UnmatchedQuote),
            }
        }
    }
    
    fn read_param(&mut self, first: char) -> Result<Token, ReadError> {
        let mut name = Name::new(first)?;

        // First we get the token
        while let Some(char) = self.chars.pop() {
            match char {
                c if c.is_whitespace() => break,

                // The next thing is a delimiter, so we have a key
                ':' => return Ok(Token::Key(name)),

                // The next thing is a quote, so we have a locale
                '\"' => {
                    self.buffer = Some('\"');
                    return Ok(Token::Locale(name));
                },

                // Any valid char is added to the buffer, unchecked.
                c => name.add(c)?,
            }
        }

        // If we got here, there was a whitespace
        loop {
            match self.chars.pop() {
                // Eat the space
                Some(c) if c.is_whitespace() => {},

                // The next thing is a delimiter, so we have a key
                Some(':') => return Ok(Token::Key(name)),

                // The next thing is a quote, so we have a locale
                Some('\"') => {
                    self.buffer = Some('\"');
                    return Ok(Token::Locale(name));
                },

                // Other chars are suspicious
                Some(c) => return Err(ReadError::InvalidChar(c)),

                None => return Err(ReadError::EOF),
            }
        }
    }
}
impl Iterator for CharReader {
    type Item = Result<Token, ReadError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let c = match self.buffer.take() {
                Some(c) => Some(c),
                None => self.chars.pop(),
            };

            return match c? {
                '#' => Some(Ok(self.read_comment())),
                '!' => Some(Ok(self.read_config())),
                '"' => Some(self.read_value()),

                c if c.is_whitespace() => continue,

                c if Name::is_valid(c) => Some(self.read_param(c)),

                c => Some(Err(ReadError::InvalidChar(c))),
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Config(String),
    Comment(String),
    
    Key(Name),
    Locale(Name),

    Value(String),
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(c) => write!(f, "Config({c})"),
            Self::Comment(c) => write!(f, "Comment({c})"),
            Self::Key(name) => write!(f, "Key({name})"),
            Self::Locale(name) => write!(f, "Locale({name})"),
            Self::Value(v) => write!(f, "Value({v})"),
        }
    }
}
