use crate::reader::ReadError;

#[cfg(test)]
mod tests;

/// Just a string where every char is guaranteed to be valid.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Name(String);
impl Name {
    /// Creates a new name. Note that an empty name is not valid, and so the 
    /// first (or only) char must be given.
    /// 
    /// # Errors
    /// If any char is invalid.
    /// 
    /// # Notes
    /// Allocates for 5 characters, as that is enough for most regular locales,
    /// e.g. "en-uk".
    pub fn new(first: char) -> Result<Self, ReadError> { 
        let mut inner = String::with_capacity(5);
        inner.push(Self::validate_first_char(first)?);

        Ok(Self(inner))
    }

    #[must_use]
    pub fn to_str(&self) -> &str { &self.0 }

    /// Adds a char.
    /// 
    /// # Errors
    /// If the char is not valid.
    pub fn add(&mut self, char: char) -> Result<(), ReadError> {
        if self.0.is_empty() {
            Self::validate_first_char(char).map(|c| self.0.push(c))
        } else {
            Self::validate_char(char).map(|c| self.0.push(c))
        }
    }

    const fn validate_char(c: char) -> Result<char, ReadError> {
        match c {
            '0'..='9' | 'a'..='z' => Ok(c),
            
            'A'..='Z' => Ok(c.to_ascii_lowercase()),
            
            '_' | '-' => Ok('_'),

            c => Err(ReadError::NameInvalid(c)),
        }    
    }

    const fn validate_first_char(c: char) -> Result<char, ReadError> {
        match c {
            'a'..='z' => Ok(c),
            
            'A'..='Z' => Ok(c.to_ascii_lowercase()),
            
            c => Err(ReadError::NameInvalidFirst(c)),
        }    
    }

    #[must_use]
    /// Gives a name suitable for a type or variant
    pub fn type_name(&self) -> String {
        self.0
        .split('_')
        .filter_map(|p| {
            // Capitalise first letter
            let mut cs = p.chars();
            cs.next().map(|c| 
                String::from(c.to_ascii_uppercase()) 
                + &cs.collect::<String>()
            )
        })
        .collect()
    }
    
    pub(crate) const fn is_valid(c: char) -> bool {
        Self::validate_char(c).is_ok()
    }
}
impl From<Name> for String {
    fn from(value: Name) -> Self { value.0 }
}
impl AsRef<str> for Name {
    fn as_ref(&self) -> &str { &self.0 }
}
impl TryFrom<&str> for Name {
    type Error = ReadError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // No char is less than a byte... so this is never too little
        let len = value.len();
        if len == 0 {
            return Err(ReadError::EmptyName);
        }

        // would allocate the correct number, but probably takes longer
        // let len = value.chars().count();
        let mut name = Self(String::with_capacity(len));
        for c in value.chars() { name.add(c)?; }
        Ok(name)
    }
}
