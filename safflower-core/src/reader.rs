use std::str::Chars;
use crate::is_valid_char;

#[cfg(test)]
mod tests;

pub struct CharReader<'a> {
    chars: Chars<'a>,
    buffer: Option<char>,
}
impl<'a> CharReader<'a> {
    #[must_use]
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars(),
            buffer: None,
        }
    }

    fn read_comment(&mut self) -> Token {
        let mut comment = String::new();
        for c in self.chars.by_ref() {
            if c == '\n' { break; }
            comment += &c.to_string();
        }
        Token::Comment(comment)
    }
    
    fn read_config(&mut self) -> Token {
        let mut line = String::new();
        let mut add = true;

        for c in self.chars.by_ref() {
            if c == '\n' { break; }
            if c == '#' { add = false; }
 
            if add { line += &c.to_string(); }
        }
        Token::Config(line)
    }
    
    fn read_value(&mut self) -> Token {
        let mut value = String::new();
        loop {
            match self.chars.next() {
                Some('"') => if value.ends_with('\\') {
                    // Quote may be escaped...
                    _ = value.pop();
                    value += "\"";
                } 
                else {
                    // ..or end the value
                    return Token::Value(value)
                },

                // Any random char is added to the buffer, unchecked.
                Some(c) => value += &c.to_string(),

                // If the iterator finished without closing the quote, you have
                // some problems in your file.
                None => panic!("unexpected EOF before terminating quote"),
            }
        }
    }
    
    fn read_param(&mut self, first: char) -> Token {
        assert!(is_valid_char(first), "unexpected char '{first}'");

        let mut value = String::from(first);

        // First we get the token
        loop {
            match self.chars.next() {
                Some(c) if c.is_whitespace() => break,

                // Any valid char is added to the buffer, unchecked.
                Some(c) if is_valid_char(c) => value += &c.to_string(),

                Some(':') => return Token::Key(value),

                // Other chars are suspicious
                Some(c) => panic!("unexpected char '{c}'"),

                None => break,
            }
        }

        // If we got here, there was a whitespace
        loop {
            match self.chars.next() {
                // Eat the space
                Some(c) if c.is_whitespace() => {},

                // The next thing is a delimiter, so we have a key
                Some(':') => return Token::Key(value),

                // The next thing is a quote, so we have a locale
                Some('\"') => {
                    self.buffer = Some('\"');
                    return Token::Locale(value)
                },

                // Other chars are suspicious
                Some(c) => panic!("unexpected char '{c}'"),

                None => panic!("unexpected EOF"),
            }
        }
    }
}

impl Iterator for CharReader<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let c = match self.buffer.take() {
                Some(c) => Some(c),
                None => self.chars.next(),
            };

            match c {
                Some(c) if c.is_whitespace() => {},

                None =>      return None,
                Some('#') => return Some(self.read_comment()),
                Some('!') => return Some(self.read_config()),
                Some('"') => return Some(self.read_value()),
                Some(c) =>   return Some(self.read_param(c)),
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Config(String),
    Comment(String),
    
    Key(String),
    Locale(String),

    Value(String),
}
