use proc_macro2::{Span, TokenStream};
use quote::quote;

use crate::{error::Error, name::Name, parser::{Key, ParseError, config::Configuration, key::KeyBuilder}};

#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct Scope {
    pub keys: Vec<Key>,
    pub nested: Vec<(Name, Box<Scope>)>,
}
impl Scope {
    pub(crate) fn generate_entries(&self) -> TokenStream {
        let keys = self.keys
        .iter()
        .map(Key::generate)
        .collect::<Vec<_>>();

        let nested = self.nested
        .iter()
        .map(|(name, scope)| {
            let inner = scope.generate_entries();
            let module = syn::Ident::new(name.as_str(), Span::call_site());
            quote! {
                pub mod #module { 
                    use super::Locale;
                    #inner 
                }
            }
        });

        quote! {
            #(#keys)*
            #(#nested)*
        }
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct ScopeBuilder {
    pub keys: Vec<KeyBuilder>,
    pub nested: Vec<(Name, Box<ScopeBuilder>)>,
}
impl ScopeBuilder {
    pub fn build(
        self, 
        config: &Configuration,
    ) -> Result<Scope, Error> {
        let Self { keys, nested } = self;
        
        let keys = keys
        .into_iter()
        .map(|k| k.build(config.locales()))
        .collect::<Result<_,_>>()
        .map_err(|e| config.parse_err(e))?;

        let nested = nested
        .into_iter()
        .map(|(name, ts)| ts
            .build(config)
            .map(|s| (name, Box::new(s)))
        )
        .collect::<Result<_,_>>()?;

        Ok(Scope {
            keys,
            nested,
        })
    }
    
    pub(crate) fn add_key(&mut self, key: KeyBuilder) -> Result<(), ParseError> {
        // Check if an old key matches the new one
        let Some(old) = self.keys
        .iter_mut()
        .find(|k| k.id() == key.id()) else {
            self.keys.push(key);
            return Ok(());
        };
        
        old.consume(key)
    }
    
    pub(crate) fn traverse (&mut self, name: Name) -> &mut Self {
        let index = self.nested
        .iter_mut()
        .position(|(n, _)| n == &name)
        .unwrap_or_else(|| {
            let i = self.nested.len();
            self.nested.push((name, Box::default()));
            i
        });

        unsafe{ 
            &mut self.nested.get_unchecked_mut(index).1
        }

    }
}
