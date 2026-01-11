use crate::{error::Error, name::Name, parser::{Key, TempKey, config::Configuration}};

#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct Scope {
    pub keys: Vec<Key>,
    pub nested: Vec<(Name, Box<Scope>)>,
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct TempScope {
    pub keys: Vec<TempKey>,
    pub nested: Vec<(Name, Box<TempScope>)>,
}
impl TempScope {
    pub fn validate(
        self, 
        config: &Configuration,
    ) -> Result<Scope, Error> {
        let Self { keys, nested } = self;
        
        let keys = keys
        .into_iter()
        .map(|k| k.validate(&config.locales).map_err(|e| config.parse_err(e)))
        .collect::<Result<_,_>>()?;

        let nested = nested
        .into_iter()
        .map(|(name, ts)| ts
            .validate(config)
            .map(|s| (name, Box::new(s)))
        )
        .collect::<Result<_,_>>()?;

        Ok(Scope {
            keys,
            nested,
        })
    }
}
impl Scope {
    pub fn create(
        config: &Configuration,
        keys: Vec<TempKey>,
    ) -> Result<Self, Error> {
        let mut root = Self {
            keys: Vec::new(),
            nested: Vec::new(),
        };

        let (root_keys, mut rest): (Vec<_>, _) = keys
        .into_iter()
        .partition(|k| k.scope.is_empty());

        root.keys = root_keys
        .into_iter()
        .map(|k| k
            .validate(&config.locales)
            .map_err(|e| config.parse_err(e))
        )
        .collect::<Result<_,_>>()?;

        loop {
            if rest.is_empty() { return Ok(root); }

            // We have some scopes
            let mut new = rest.remove(0);
            let scope = new.scope.remove(0);

            // Get all that match!
            dbg!(rest.iter().map(|s| &s.scope).collect::<Vec<_>>());

            let new_keys = rest.extract_if(
                .., 
                |k| k.scope
                    .first()
                    .expect("the scope wasn't empty, but is now...") 
                    == &scope
            )
            .map(|mut k| {
                k.scope.remove(0);
                k
            })
            .collect::<Vec<_>>();

            let nested = Scope::create(config, new_keys)?;
            dbg!(&scope);
            dbg!(&nested);

            root.nested.push((scope, Box::new(nested)));
        }
    }
}
