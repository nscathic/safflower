use crate::{name::Name, parser::ParseError, shorten};

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Key {
    pub id: Name,
    pub arguments: Vec<String>,
    pub comment: Option<String>,
    pub entries: Vec<String>,
}
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

    // pub fn scope(self, scope: &str) -> Self {
    //     let Self { id, arguments, comment, entries, .. } = self;
    //     Self {
    //         id,
    //         scope: Some(scope.to_string()),
    //         arguments,
    //         comment,
    //         entries,
    //     }
    // }

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

    pub fn entries(self, entries: &[&str]) -> Self {
        let Self { id, arguments, comment, .. } = self;
        Self {
            id,
            arguments,
            comment,
            entries: entries.iter().map(ToString::to_string).collect(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Entry {
    pub value: String,
    pub comment: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TempKey {
    pub id: Name,
    pub scope: Vec<Name>,
    pub comment: Option<String>,
    pub entries: Vec<Option<Entry>>,
}
impl TempKey {
    /// Validates and compiles data.
    /// 
    /// # Errors 
    /// Returns errors for badly formatted data.
    pub fn validate(self, locales: &[Name]) -> Result<Key, ParseError> {
        if locales.is_empty() { return Err(ParseError::NoLocales); }
        
        let Self { id, comment, entries, .. } = self;

        let (entries, comments) = get_entries(entries, &id, locales)?;
        let comment = get_comment(comments, comment, locales);
        let arguments = get_arguments(&entries, &id, locales)?;

        Ok(Key {
            id,
            arguments,
            comment,
            entries,
        })
    }
}

fn get_arguments(
    entries: &[String], 
    id: &Name,
    locales: &[Name],
) -> Result<Vec<String>, ParseError> {
    let arguments = extract_arguments(&entries[0])?;
        
    let mismatch = entries
    .iter()
    .enumerate()
    .skip(1)
    .map(|(i, e)| (i, extract_arguments(e)))
    .find(|(_, a)| !a.as_ref().is_ok_and(|a| a == &arguments));

    if let Some((index, result)) = mismatch {
        let args = result?;
        return Err(ParseError::ArgumentMismatch(
            id.to_str().to_string(), 
            locales[index].to_str().to_string(),
            args,
            arguments,
        ));
    }

    Ok(arguments)
}

fn extract_arguments(key: &str) -> Result<Vec<String>, ParseError> {
    let mut arguments = Vec::new();
    let mut argument = String::new();
    let mut opened = false;
    let mut unnamed_indexer = 0;
    let mut formatting = false;

    for c in key.chars() {
        match c {
            '{' if opened => return Err(ParseError::NestedBrace),
            '{' => { opened = true; },

            '}' if !opened => return Err(ParseError::ExtraClosingBrace),
            '}' => {
                if argument.is_empty() {
                    argument = format!("{unnamed_indexer}");
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
                Name::validate_char(c)
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
    locales: &[Name],
) -> Option<String> {
    let locale_comment = comments
    .into_iter()
    .enumerate()
    .filter_map(|(i, comment)| 
        comment.map(|c| format!("- *{}*: {c}\n", locales[i].to_str()))
    )
    .collect::<String>();

    if locale_comment.is_empty() { return key_comment; }
    
    Some(format!(
        "{} # Locale notes\n{locale_comment}", 
        key_comment.unwrap_or_default(),
    ))
}

fn get_entries(
    entries: Vec<Option<Entry>>,
    id: &Name,
    locales: &[Name],
) -> Result<(Vec<String>, Vec<Option<String>>), ParseError> {
    if entries.len() < locales.len() {
        return Err(ParseError::EntryMissingLocale(
            shorten(id), 
            locales[entries.len()].to_string(),
        ));
    }

    entries
    .into_iter()
    .enumerate()
    .map(|(i, e)| e.ok_or_else(|| ParseError::EntryMissingLocale(
        shorten(id),
        locales[i].to_str().to_string()
    )))
    .collect::<Result<Vec<Entry>,_>>()
    .map(|ok| ok
        .into_iter()
        .map(|e| (e.value, e.comment))
        .unzip()
    )
}
