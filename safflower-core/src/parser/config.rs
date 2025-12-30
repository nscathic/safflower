use crate::{error::Error, name::Name, parser::ParseError};

#[derive(Default)]
pub struct Configuration {
    pub locales: Vec<Name>,
}
impl Configuration {
    /// Parses a config line.
    /// 
    /// # Errors
    /// If the line is empty or contains an unrecognised key, or if there is 
    /// an error in the specific command.
    pub fn parse_config(&mut self, line: &str) -> Result<(), Error> {
        let mut parts = line.split_whitespace();
        let Some(key) = parts.next() else { 
            return Err(ParseError::EmptyKey.into())
        };

        let values = parts.collect::<Vec<_>>();

        match key {
            "locales" => self.set_locales(values),
            _ => Err(ParseError::UnknownKey((*key).to_string()).into()),
        }
    }

    pub fn find_locale(&self, locale: &Name) -> Option<usize> {
        self.locales
        .iter()
        .position(|l| l == locale)
    }
    
    /// # Errors
    /// Not having any locales, or inserting the same value twice.
    pub fn set_locales(
        &mut self, 
        parts: Vec<&str>,
    ) -> Result<(), Error> {
        if parts.is_empty() { 
            return Err(ParseError::MissingValues("locales").into()); 
        }

        for part in parts {
            let locale = Name::try_from(part)?;

            if self.locales.iter().any(|l| l==&locale) {
                return Err(ParseError::DuplicateLocale(locale.into()).into());
            }

            self.locales.push(locale);
        }

        Ok(())
    }
}
