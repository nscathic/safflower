use std::{matches, rc::Rc};

use crate::reader::ReadError;

#[cfg(test)]
mod tests;

/// Just a string where every char is guaranteed to be valid.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Name(Rc<[ValidChar]>);
impl Name {
    #[must_use]
    pub fn as_str(&self) -> &str { 
        unsafe { 
            // The original data is a vector of u8 aliases, and due to the free 
            // abstraction we can just cast it as a vector of u8s.
            let slice = std::slice::from_raw_parts(
                self.0.as_ptr().cast(),
                self.0.len(),
            );
            str::from_utf8_unchecked(slice)
        }
    }

    #[must_use]
    /// Generates a `String` suitable for a type or variant.
    pub fn type_name(&self) -> String {
        let mut new = Vec::with_capacity(self.0.len());
        let mut capitalise_next = true;
        for c in self.0.iter() {
            if c.0 == b'_' {
                capitalise_next = true;
                continue;
            }

            if capitalise_next {
                new.push(c.to_u8_capitalised());
                capitalise_next = false;
            } else {
                new.push(c.0);
            }
        }
        
        unsafe { String::from_utf8_unchecked(new) }
    }
}
impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
impl AsRef<str> for Name { 
    fn as_ref(&self) -> &str { self.as_str() }
}
impl TryFrom<&str> for Name {
    type Error = ReadError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // No char is less than a byte... so this is never too little
        let len = value.len();
        if len == 0 { return Err(ReadError::EmptyName); }

        // would allocate the correct number, but probably takes longer
        // let len = value.chars().count();
        let mut name = NameBuilder(Vec::with_capacity(len));
        for c in value.chars() { name.add(c)?; }
        Ok(name.build())
    }
}

pub struct NameBuilder(Vec<ValidChar>);
impl NameBuilder {
    /// Creates a bulider for a name. Note that an empty name is not valid, and 
    /// so the first (or only) char must be given.
    /// 
    /// # Errors
    /// If any char is invalid.
    /// 
    /// # Notes
    /// Allocates for 5 characters from the start, as that is enough for most 
    /// regular locales, e.g. "en-uk".
    pub fn new(first: char) -> Result<Self, ReadError> { 
        let mut inner = Vec::with_capacity(5);
        inner.push(ValidChar::validate(first, true)?);

        Ok(Self(inner))
    }

    /// Adds a char, if it is valid.
    /// 
    /// # Errors
    /// If the char is not valid.
    pub fn add(&mut self, char: char) -> Result<(), ReadError> {
        let c = ValidChar::validate(
            char, 
            self.0.is_empty(),
        )?;

        self.0.push(c);
        Ok(())
    }

    #[must_use]
    pub fn build(self) -> Name {
        Name(self.0.into())
    }
}

/// Represent a valid name character, i.e. a digit, lowercase latin character, 
/// or `_`.
/// 
/// Needless to say, this is also guaranteed valid UTF-8, so we can safely 
/// handle some naming functions more easily.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ValidChar(u8);
impl ValidChar {
    /// Returns the lowercase version of any supplied char.
    /// 
    /// # Errors 
    /// If the char cannot be made valid.
    #[allow(
        clippy::as_conversions,
        clippy::arithmetic_side_effects,
    )]
    pub(crate)  const fn validate(c: char, first: bool) -> Result<Self, ReadError> {
        match c {
            'a'..='z' => Ok(Self(c as u8)),
            'A'..='Z' => Ok(Self(c as u8 + CAPITALISATION)),

            '0'..='9' if !first => Ok(Self(c as u8)), 
            '_' | '-' if !first => Ok(Self(b'_')),
            
            c => Err(ReadError::NameInvalid(c)),
        }    
    }

    #[must_use]
    pub(crate) const fn is_valid(c: char) -> bool {
        matches!(
            c, 
            '0'..='9' | 
            'a'..='z' |
            'A'..='Z' |
            '_' | '-' 
        )
    }
    
    #[allow(
        clippy::as_conversions,
        clippy::arithmetic_side_effects,
    )]
    const fn to_u8_capitalised(self) -> u8 {
        match self.0 {
            b'a'..=b'z' => self.0 - CAPITALISATION,
            c => c
        }
    }
    
    #[allow(clippy::as_conversions)]
    pub(crate) const fn to_char(self) -> char { self.0 as char }
}

const CAPITALISATION: u8 = b'a' - b'A';
