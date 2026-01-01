use std::path::PathBuf;

use crate::{error::Error, name::Name, parser::ParseError};

pub struct Configuration {
    pub current_path: PathBuf,
    pub locales: Vec<Name>,
    pub path_queue: Vec<PathBuf>,
}
impl Configuration {
    pub const fn new(root: PathBuf) -> Self {
        Self { 
            current_path: root, 
            locales: Vec::new(),
            path_queue: Vec::new(),
        }
    }
    
    /// Parses a config line.
    /// 
    /// # Errors
    /// If the line is empty or contains an unrecognised key, or if there is 
    /// an error in the specific command.
    pub fn parse_config(&mut self, line: &str) -> Result<(), Error> {
        let mut parts = line.split_whitespace();
        let Some(key) = parts.next() else { 
            return Err(Error::Parse(
                self.current_path.clone(),
                ParseError::ConfigEmptyKey,
            ))
        };

        let values = parts.collect::<Vec<_>>();

        match key {
            "locales" => self.set_locales(values)?,
            "inlcude" => self.queue_path(values),
            k => return Err(Error::Parse(
                self.current_path.clone(),
                ParseError::ConfigUnknownKey(k.to_string()),
            )),
        }

        Ok(())
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
            return Err(Error::Parse(
                self.current_path.clone(),
                ParseError::ConfigMissingValues("locales")
            )); 
        }

        for part in parts {
            let locale = Name::try_from(part)?;

            if self.locales.iter().any(|l| l==&locale) {
                return Err(Error::Parse(
                    self.current_path.clone(),
                    ParseError::DuplicateLocale(locale.into())
                ));
            }

            self.locales.push(locale);
        }

        Ok(())
    }
    
    /// The number of locales declared.
    pub const fn locale_count(&self) -> usize { self.locales.len() }
    
    fn queue_path(&mut self, values: Vec<&str>) {
        let parent = self.current_path.parent();
        let new_paths = values
        .into_iter()
        .rev()
        .map(PathBuf::from)
        .map(|path| 
            if let Some(parent) = parent {
                PathBuf::from(parent).join(path)
            } else { 
                path
            }
        );

        let old_queue = std::mem::take(&mut self.path_queue);
        self.path_queue = new_paths.chain(old_queue).collect();
    }
    
    pub fn pop_path(&mut self) -> Option<PathBuf> {
        let path = self.path_queue.pop();

        if let Some(path) = &path {
            self.current_path.clone_from(path);
        }

        path
    }
}
