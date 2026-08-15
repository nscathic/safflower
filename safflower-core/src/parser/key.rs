use std::rc::Rc;

use proc_macro2::{Span, TokenStream};
use quote::quote;

use crate::{name::{Name, ValidChar}, parser::{ParseError, config::Locale}, shorten};

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct KeyBuilder {
    id: Name,
    comment: Option<String>,
    entries: Vec<Entry>
}
impl KeyBuilder {
    pub(crate) const fn new(id: Name, comment: Option<String>) -> Self {
        Self { 
            id, 
            comment,
            entries: Vec::new(),
        }
    }
    
    /// Returns `true` for a collision.
    pub(crate) fn add_entry(
        &mut self, 
        locale: Rc<Locale>, 
        value: String, 
        comment: Option<String>,
    ) -> bool {
        if self.entries.iter().any(|e| e.locale == locale) { return true; }
        
        self.entries.push(Entry {
            locale, 
            value, 
            comment,
        });

        false
    }
    
    pub(crate) fn build(
        self, 
        locales: &[Rc<Locale>],
    ) -> Result<Key, ParseError> {
        if locales.is_empty() { return Err(ParseError::NoLocales); }
        
        let Self { id, comment, mut entries } = self;

        if entries.len() < locales.len() {
            return Err(ParseError::EntryMissingLocale(
                shorten(id), 
                unsafe { 
                    locales
                    .get_unchecked(entries.len())
                    .key()
                    .to_string()
                },
            ));
        }

        let comments = entries
        .iter_mut()
        .map(|e| e.comment.take())
        .collect();

        let comment = get_comment(comments, comment);
        let arguments = get_arguments(&entries, &id, locales)?;

        Ok(Key {
            id,
            arguments,
            comment,
            entries,
        })
    }
    
    pub(crate) fn consume(&mut self, other: Self) -> Result<(), ParseError> {
        let Self { comment, mut entries, .. } = other;

        // Check for overlaps
        if let Some(entry) = entries
        .iter()
        .find(|e| 
            self.entries
            .iter()
            .any(|se| se.locale == e.locale)
        ) {
            return Err(ParseError::DuplicateEntry(
                self.id.to_string(),
                entry.locale.key().to_string(),
            ));
        }
        
        self.entries.append(&mut entries);

        let Some(c2) = comment else { return Ok(()) };

        if let Some(c1) = self.comment.as_mut() {
            c1.push_str(&c2);
        } else {
            self.comment = Some(c2);
        }

        Ok(())
    }
    
    pub fn id(&self) -> &str { self.id.as_ref() }

}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Entry {
    pub locale: Rc<Locale>,
    pub value: String,
    pub comment: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Key {
    pub id: Name,
    pub arguments: Vec<String>,
    pub comment: Option<String>,
    pub entries: Vec<Entry>,
}
impl Key {
    #[must_use]
    pub fn generate(&self) -> TokenStream {
        // Simpler case for const strings
        if self.arguments.is_empty() {
            return self.generate_const();
        }
        
        // All go to params, but only positinal go to arguments
        let (positional, named): (Vec<_>, Vec<_>) = self.arguments
        .iter()
        .partition(|a| a.chars().all(char::is_numeric));

        let named = named
        .iter()
        .map(|a| syn::Ident::new(a, Span::call_site()));

        let positional = positional
        .iter()
        .map(|i| format!("arg{i}"))
        .map(|a| syn::Ident::new(&a, Span::call_site()));

        let arguments = positional.clone().collect::<Vec<_>>();
        let params = named.chain(positional);

        let id = syn::Ident::new(self.id.as_str(), Span::call_site());
        let comment = self.comment.as_ref().map(|c| quote! {#[doc = #c]});

        let entries = self.entries
        .iter()
        .map(|entry| {
            let locale = &entry.locale.ident();
            let value = &entry.value;
            quote! {
                Locale::#locale => format!(#value, #(#arguments,)*)
            }
        });

        quote! {
            #comment
            pub fn #id(
                locale: Locale,
                #(#params:impl std::fmt::Display,)*
            ) -> String {
                match locale {
                    #(#entries,)*
                }
            }
        }
    }

    fn generate_const(&self) -> TokenStream {
        let id = syn::Ident::new(self.id.as_str(), Span::call_site());
        let comment = self.comment.as_ref().map(|c| quote! {#[doc = #c]});

        let entries = self.entries
        .iter()
        .map(|entry| {
            let locale = entry.locale.ident();
            let value = &entry.value;
            quote! {
                Locale::#locale => #value
            }
        });

        quote! {
            #comment
            pub const fn #id(locale: Locale) -> &'static str {
                match locale {
                    #(#entries,)*
                }
            }
        }
    }
}
#[allow(
    clippy::unwrap_used,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
)]
#[cfg(test)]
impl Key {
    pub fn name(id: &str) -> Self {
        Self {
            id: Name::try_from(id).unwrap(),
            arguments: Vec::new(),
            comment: None,
            entries: Vec::new(),
        }
    }

    pub fn arguments(self, arguments: &[&str]) -> Self {
        let Self { id, comment, entries,.. } = self;
        Self {
            id,
            arguments: arguments.iter().map(ToString::to_string).collect(),
            comment,
            entries,
        }
    }

    pub fn comment(self, comment: &str) -> Self {
        let Self { id, arguments, entries, .. } = self;
        Self {
            id,
            arguments,
            comment: Some(comment.to_string()),
            entries,
        }
    }

    pub fn entries(self, entries: &[(&Rc<Locale>, &str)]) -> Self {
        let Self { id, arguments, comment, .. } = self;
        let entries = entries
        .iter()
        .map(|(loc, val)| Entry {
            locale: (*loc).clone(),
            value: (*val).to_string(),
            comment: None,
        })
        .collect();
    
        Self {
            id,
            arguments,
            comment,
            entries,
        }
    }
}

fn get_arguments(
    entries: &[Entry], 
    id: &Name,
    locales: &[Rc<Locale>],
) -> Result<Vec<String>, ParseError> {
    let arguments = extract_arguments(
        &entries.first()
        .ok_or_else(|| ParseError::NoEntries(id.to_string()))?
        .value
    )?;
        
    let mismatch = entries
    .iter()
    .enumerate()
    .skip(1)
    .map(|(i, e)| (i, extract_arguments(&e.value)))
    .find(|(_, a)| !a.as_ref().is_ok_and(|a| a == &arguments));

    if let Some((index, result)) = mismatch {
        let args = result?;
        return Err(ParseError::ArgumentMismatch(
            id.to_string(), 
            locales
                .get(index)
                .map_or_else(
                    || String::from("[unknown locale]"),
                    |l| l.key().to_string(),
                ),
            args,
            arguments,
        ));
    }

    Ok(arguments)
}

#[allow(clippy::arithmetic_side_effects)]
fn extract_arguments(key: &str) -> Result<Vec<String>, ParseError> {
    let mut arguments = Vec::new();
    let mut argument = String::new();
    let mut opened = false;
    let mut unnamed_indexer: usize = 0;
    let mut formatting = false;

    for c in key.chars() {
        match c {
            '{' if opened => return Err(ParseError::NestedBrace),
            '{' => { opened = true; },

            '}' if !opened => return Err(ParseError::ExtraClosingBrace),
            '}' => {
                if argument.is_empty() {
                    argument = unnamed_indexer.to_string();
                    unnamed_indexer += 1;
                }
                else if !argument.starts_with(
                    |c: char| c.is_ascii_alphabetic()
                ) && !argument.chars().all(char::is_numeric)  {
                    return Err(ParseError::ArgBadStart(
                        key.to_string(), 
                        shorten(&argument), 
                        c,
                    ))
                }

                if !arguments.contains(&argument) {                        
                    arguments.push(argument);
                }

                argument = String::new();
                opened = false;
                formatting = false;
            }

            ':' if opened => formatting = true,

            // Don't copy the formatting part
            c if opened && !formatting => argument.push(
                ValidChar::validate(c, false)
                .map(ValidChar::to_char)
                .map_err(|_| ParseError::ArgBadChar(
                    shorten(key), 
                    shorten(&argument),
                    c,
                ))?
            ),
            
            _ => (),
        }
    }

    Ok(arguments) 
}

fn get_comment(
    comments: Vec<Option<String>>,
    key_comment: Option<String>,
) -> Option<String> {
    let locale_comment = comments
    .into_iter()
    .flatten()
    .collect::<String>();

    if locale_comment.is_empty() { return key_comment; }
    
    Some(key_comment
    .map_or_else(
        || format!(" # Locale notes\n{locale_comment}"),
        |kc| format!("{kc}\n\n # Locale notes\n{locale_comment}"),
    ))
}
