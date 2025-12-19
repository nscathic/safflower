use crate::{
    error::ParseError, reader::Token, shorten, validate_char
};

#[cfg(test)]
mod tests;

#[derive(Default)]
pub struct Parser {
    head: Head,
    keys: Vec<Key>,
}
impl Parser {
    /// Parses a token iterator.
    /// 
    /// # Errors 
    /// If something is unparsable.
    pub fn parse(
        &mut self, 
        tokens: impl Iterator<Item = Token>,
    ) -> Result<(), ParseError> {
        let mut comment = None;
        let mut key: Option<TempKey> = None;
        let mut locale = None;

        let mut locale_style = None;

        for token in tokens { match token {
            Token::Config(c) => {
                self.head.parse_config(&c)?;

                // Check if it was locales
                if c.starts_with("locales") {
                    locale_style = Some(self.head.locales.len());
                }
            },

            // This will get put on whatever is next
            Token::Comment(c) => comment = Some(c),
        
            Token::Key(k) => self.parse_key(
                &mut key, 
                comment.take(), 
                locale_style, 
                k.to_lowercase(),
            )?,

            Token::Locale(l) => self.parse_locale(
                &mut locale, 
                key.is_some(), 
                l.to_lowercase(),
            )?,

            Token::Value(v) => self.parse_value(
                &mut key, 
                comment.take(), 
                locale, 
                locale_style, 
                v,
            )?,
        }}

        // Push the last dangling key
        if let Some(key) = key.take() {
            self.keys.push(key.validate(&self.head.locales)?);
        }
        
        Ok(())
    }

    fn parse_locale(
        &self, 
        locale: &mut Option<usize>, 
        has_key: bool, 
        l: String,
    ) -> Result<(), ParseError> {
        if has_key {
            *locale = Some(
                self.head
                .find_locale(&l)
                .ok_or(ParseError::UndeclaredLocale(l))?
            );
        } else {
            return Err(ParseError::LocaleNoKey);
        }

        Ok(())
    }
    
    fn parse_key(
        &mut self, 
        key: &mut Option<TempKey>, 
        comment: Option<String>, 
        locale_style: Option<usize>, 
        k: String,
    ) -> Result<(), ParseError> {
        if let Some(old_key) = key.take() {
            if self.keys.iter().any(|k| k.id == old_key.id) {
                return Err(ParseError::DuplicateKey(old_key.id));
            }
            self.keys.push(old_key.validate(&self.head.locales)?);
        }
        let locale_count = locale_style.map_or(1, |c| c);
        
        *key = Some(TempKey {
            id: k,
            comment,
            entries: vec![None; locale_count],
        });
        Ok(())
    }

    fn parse_value(
        &self,
        key: &mut Option<TempKey>, 
        comment: Option<String>, 
        locale: Option<usize>, 
        locale_style: Option<usize>, 
        v: String,
    ) -> Result<(), ParseError> {
        let Some(key) = key else { return Err(ParseError::ValueNoKey) };

        let index = match locale_style {
            None => 0,
            Some(_) => locale.ok_or_else(|| 
                ParseError::UsingDefaultLocale(
                    key.id.clone(),
                    shorten(&v),
                )
            )?,
        };

        if key.entries[index].is_some() {
            let locale = self.head.locales[index].clone();
            return Err(ParseError::DuplicateEntry(
                locale,
                key.id.clone()
            ));
        }

        key.entries[index] = Some(Entry {
            value: v,
            comment,
        });
        
        Ok(())
    }
    
    #[must_use]
    pub fn extract(self) -> (Head, Vec<Key>) {
        (self.head, self.keys)
    }
}

#[derive(Default)]
/// Keeps configurational information.
pub struct Head {
    locales: Vec<String>,
}
impl Head {
    /// Parses a config line.
    /// 
    /// # Errors
    /// If the line is empty or contains an unrecognised key, or if there is 
    /// an error in the specific command.
    pub fn parse_config(&mut self, line: &str) -> Result<(), ParseError> {
        if line.is_empty() { return Err(ParseError::EmptyKey); }

        let parts = line.split_whitespace().collect::<Vec<_>>();
        let key = parts[0];

        match key {
            "locales" => self.set_locales(&parts[1..]),
            _ => Err(ParseError::UnknownKey((*key).to_string())),
        }
    }

    fn find_locale(&self, locale: &str) -> Option<usize> {
        self.locales
        .iter()
        .position(|l| l == locale)
    }
    
    pub(crate) fn set_locales(
        &mut self, 
        parts: &[&str],
    ) -> Result<(), ParseError> {
        if parts.is_empty() { 
            return Err(ParseError::MissingValues("locales")); 
        }

        for l in parts {
            let locale = l.chars()
            .map(validate_char)
            .collect::<Result<String, _>>()
            .map_err(|c| ParseError::LocaleBadChar((*l).to_string(), c))?;

            if !locale.starts_with(char::is_alphabetic) {
                return Err(ParseError::LocaleBadStart((*l).to_string()));
            }

            if self.locales.iter().any(|l| l==&locale) {
                return Err(ParseError::DuplicateLocale(locale));
            }

            self.locales.push(locale);
        }

        Ok(())
    }
    
    #[must_use]
    /// Hands over all locales
    pub fn locales(self) -> Vec<String> {
        self.locales
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct TempKey {
    id: String,
    comment: Option<String>,
    entries: Vec<Option<Entry>>,
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Key {
    pub id: String,
    pub comment: Option<String>,
    pub entries: Vec<String>,
}

impl TempKey {
    fn validate(self, locales: &[String]) -> Result<Key, ParseError> {
        if locales.is_empty() { return Err(ParseError::NoLocales); }

        let entries = self.entries
        .into_iter()
        .enumerate()
        .map(|(i, e)| e.ok_or_else(|| ParseError::MissingLocale(
            shorten(&self.id),
            locales[i].clone()
        )))
        .collect::<Result<Vec<Entry>, _>>()?;

        let (entries, comments): (Vec<String>, Vec<Option<String>>) = entries
        .into_iter()
        .map(|e| (e.value, e.comment))
        .unzip();

        let locale_comment = comments
        .into_iter()
        .enumerate()
        .filter_map(|(i, comment)| 
            comment.map(|c| format!("- *{}*: {c}\n", locales[i]))
        )
        .collect::<String>();

        let comment = if locale_comment.is_empty() {
            self.comment
        }
        else {
            Some(format!(
                "{} # Locale notes\n{locale_comment}", 
                self.comment.unwrap_or_default(),
            ))
        };

        let id = self.id.chars()
        .map(validate_char)
        .collect::<Result<String, _>>()
        .map_err(|c| ParseError::KeyBadChar(self.id.clone(), c))?;
        
        if !id.starts_with(char::is_alphabetic) {
            return Err(ParseError::KeyBadStart(self.id));
        }

        Ok(Key {
            id,
            comment,
            entries,
        })
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Entry {
    pub value: String,
    pub comment: Option<String>,
}
