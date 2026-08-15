use std::{path::PathBuf, rc::Rc};

use proc_macro2::Span;

use crate::{error::Error, name::Name, parser::ParseError, reader::ReadError};

pub struct Configuration {
    locales: Vec<Rc<Locale>>,
    
    current_module: Module,
    current_scope: Option<Name>,

    queue: Vec<Module>,
}
impl Configuration {
    #[must_use]
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
    pub fn parse_config(&mut self, key: &str, args: &str) -> Result<(), Error> {
        if key.is_empty() { 
            return Err(self.parse_err(ParseError::ConfigEmptyKey))
        }

        match key {
            "locales" => self.parse_locales(args)?,
            "include" => self.parse_include(args)?,
            "scope"   => self.parse_scope(args)?,

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
    
    #[must_use]
    pub fn find_locale(&self, locale: &Name) -> Option<Rc<Locale>> {
        self.locales
        .iter()
        .find(|l| &l.key == locale)
        .cloned()
    }
    
    pub fn pop_path(&mut self) -> Option<Module> {
        let module = self.queue.pop();

        self.current_scope = None;
        if let Some(m) = &module {
            self.current_module.clone_from(m);
        }

        module
    }
    
    #[must_use]
    pub fn get_scope(&self) -> Vec<Name> { 
        let parent_scope = self.current_module.scope_chain.clone();
        
        match self.current_scope.clone() {
            Some(s) => [parent_scope, vec![s]].concat(),
            None => parent_scope,
        }
    }

    fn assert(&self, condition: bool, err: ParseError) -> Result<(), Error> {
        if condition {
            Ok(())
        } else {
            Err(self.parse_err(err))
        }
    }
    
    #[allow(clippy::arithmetic_side_effects)]
    fn parse_locales(&mut self, args: &str) -> Result<(), Error> {
        self.assert(
            !args.is_empty(), 
            ParseError::ConfigMissingValues("locales"),
        )?; 

        // We can parse either
        //     KEY
        // or 
        //     KEY(ACTUAL NAME)

        let mut current = args.trim();
        let mut end_key_index;
        let mut start_next_index;
        let mut name;// = None;

        while !current.is_empty() {
            (end_key_index, start_next_index) = current
            .find('(')
            .map(|start|
                // If there's a (, look for a )
                current
                .find(')')
                .filter(|e| e > &start)
                .map(|end| (start, end))
                .ok_or_else(||self.parse_err(
                    ParseError::UnmatchedNameParen(args.to_string())
                ))
            )
            // Filter out the error, if any
            .transpose()?
            .unwrap_or_else(|| {
                // Default case: no ()
                let i = current
                .find(char::is_whitespace)
                // To catch EOL
                .unwrap_or(current.len());

                (i, i)
            });

            name = (end_key_index != start_next_index)
            .then(|| unsafe { 
                current
                .get_unchecked(end_key_index + 1..start_next_index)
                .to_string()
            });

            let key = unsafe { current.get_unchecked(..end_key_index) };
            let key = Name::try_from(key).map_err(|e| self.read_err(e))?;
            self.assert(
                self.find_locale(&key).is_none(),
                ParseError::DuplicateLocale(key.to_string()),
            )?;
            
            current = current
            .get(start_next_index + 1..)
            .map(str::trim)
            .unwrap_or_default();
                
            self.locales.push(Rc::new(Locale::new(key, name.take())));
        }

        Ok(())
    }
    
    fn parse_include(&mut self, args: &str) -> Result<(), Error> {
        self.assert(
            !args.is_empty(),
            ParseError::ConfigMissingValues("locales")
        )?; 

        // The new scope is whatever the parent's was, plus any declared scope.
        let scope_chain = self.get_scope();

        let parent_dir = self.current_module.path.parent();
        let new_paths = args
        .split_whitespace()
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
    
    fn parse_scope(&mut self, args: &str) -> Result<(), Error> {
        let mut iter = args.split_whitespace();
        let Some(scope) = iter.next() else {
            return Err(self.parse_err(
                ParseError::ConfigMissingValues("include")
            ));
        };
        let scope = Name::try_from(scope).map_err(|e| self.read_err(e))?;

        #[allow(clippy::arithmetic_side_effects)]
        self.assert(
            iter.next().is_none(),
            ParseError::ConfigMultipleScopes(iter.count() + 2),
        )?;

        self.current_scope = Some(scope);

        Ok(())
    }
       
    pub(crate) fn locales(&self) -> &[Rc<Locale>] { &self.locales }
    
    pub(crate) fn into_locales(self) -> Vec<Rc<Locale>> { self.locales }
}

#[derive(Debug, Clone)]
pub struct Module {
    pub scope_chain: Vec<Name>,
    pub path: PathBuf,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Locale {
    key: Name, 
    name: Option<String>,
    type_name: String,
    ident: syn::Ident,
}
impl Locale {
    pub(crate) fn new(key: Name, name: Option<String>) -> Self {
        let type_name = key.type_name();
        let ident = syn::Ident::new(&type_name, Span::call_site());

        Self { key, name, type_name, ident }
    }
    
    #[must_use]
    pub const fn key(&self) -> &Name { &self.key }
    
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_deref().unwrap_or(self.key.as_str())
    }
    
    #[must_use]
    pub fn type_name(&self) -> &str { &self.type_name }
    
    #[must_use]
    pub const fn ident(&self) -> &syn::Ident { &self.ident }
}
