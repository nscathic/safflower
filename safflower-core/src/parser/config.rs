use std::path::PathBuf;

use crate::{error::Error, name::Name, parser::ParseError, reader::ReadError};

pub struct Configuration {
    pub locales: Vec<Name>,
    
    current_module: Module,
    current_scope: Option<Name>,

    queue: Vec<Module>,
}
impl Configuration {
    pub const fn new(root: PathBuf) -> Self {
        Self { 
            current_module: Module { 
                scope_chain: vec![], 
                path: root 
            },
            current_scope: None,

            locales: Vec::new(),
            queue: Vec::new(),
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
            return Err(self.parse_err(ParseError::ConfigEmptyKey))
        };

        let values = parts.collect::<Vec<_>>();

        match key {
            "locales" => self.locales(values)?,
            "include" => self.include(values)?,
            "scope"   => self.scope(&values)?,

            k => return Err(self.parse_err(
                ParseError::ConfigUnknownKey(k.to_string())
            )),
        }

        Ok(())
    }

    pub fn parse_err(&self, err: ParseError) -> Error {
        Error::Parse(self.current_module.path.clone(), err)
    }
    
    pub fn read_err(&self, err: ReadError) -> Error {
        Error::Read(self.current_module.path.clone(), err)
    }
    
    fn assert(&self, condition: bool, err: ParseError) -> Result<(), Error> {
        if condition {
            Ok(())
        } else {
            Err(self.parse_err(err))
        }
    }
    
    pub fn find_locale(&self, locale: &Name) -> Option<usize> {
        self.locales
        .iter()
        .position(|l| l == locale)
    }
    
    pub const fn locale_count(&self) -> usize { self.locales.len() }
    
    pub fn pop_path(&mut self) -> Option<Module> {
        let module = self.queue.pop();

        self.current_scope = None;
        if let Some(m) = &module {
            self.current_module.clone_from(m);
        }

        module
    }
    
    pub fn get_scope(&self) -> Vec<Name> { 
        let parent_scope = self.current_module.scope_chain.clone();
        match self.current_scope.clone() {
            Some(s) => [parent_scope, vec![s]].concat(),
            None => parent_scope,
        }
    }

    fn locales(&mut self, values: Vec<&str>) -> Result<(), Error> {
        self.assert(
            !values.is_empty(), 
            ParseError::ConfigMissingValues("locales"),
        )?; 

        for part in values {
            let locale = Name::try_from(part).map_err(|e| self.read_err(e))?;

            self.assert(
                !self.locales.iter().any(|l| l==&locale),
                ParseError::DuplicateLocale(locale.to_string()),
            )?;

            self.locales.push(locale);
        }

        Ok(())
    }
    
    fn include(&mut self, values: Vec<&str>) -> Result<(), Error> {
        self.assert(
            !values.is_empty(),
            ParseError::ConfigMissingValues("locales")
        )?; 

        // The new scope is whatever the parent's was, plus any declared scope.
        let scope_chain = self.get_scope();

        let parent_dir = self.current_module.path.parent();
        let new_paths = values
        .into_iter()
        .rev()
        .map(|path|
            parent_dir.map_or_else(
                || PathBuf::from(path),
                |parent| PathBuf::from(parent).join(path)
            ) 
        )
        .map(|path| Module { 
            scope_chain: scope_chain.clone(), 
            path
        });

        let old_queue = std::mem::take(&mut self.queue);
        self.queue = new_paths.chain(old_queue).collect();

        Ok(())
    }
    
    fn scope(&mut self, values: &[&str]) -> Result<(), Error> {
        let &[scope] = values else {
            return Err(self.parse_err(ParseError::ConfigMissingValues("include")));
        };
        let scope = Name::try_from(scope).map_err(|e| self.read_err(e))?;

        self.assert(
            values.len() == 1,
            ParseError::ConfigMultipleScopes(values.len()),
        )?;

        self.current_scope = Some(scope);

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Module {
    pub scope_chain: Vec<Name>,
    pub path: PathBuf,
}
