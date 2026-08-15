# `safflower` 
*Statically-Allocated Fromat-Friendly Localising (Written Entirely in Rust)*

[![Webiste](https://img.shields.io/badge/site-nscathic-blue)](https://nscathic.eu/projects/safflower.html)
[![Crates.io Version](https://img.shields.io/crates/v/safflower)](https://crates.io/crates/safflower)
[![docs](https://img.shields.io/docsrs/safflower?logo=rust)](https://docs.rs/safflower/latest/)
[![Repo](https://img.shields.io/badge/github-repo-blue?logo=github)](https://github.com/nscathic/safflower)
[![License](https://img.shields.io/badge/license-MIT-blue)](license.md)

`safflower` aims to provide a no-fuss text localiser with minimum runtime overhead.
It does so in two main ways: 
1) all the text is read, checked, and parsed at compile time; and
2) all localisations of the text are available at the same time.

Below is a description of how it works. If you just want to see it in use, skip to [Accessing text](#accessing-text).

## Loading
The `load!` macro reads a text file from the path provided. If there are any errors in the file (mainly from formatting), they are caught at compile time. This removes the need for runtime error handling (path exists, file can be read, contents can be parsed, key exists, etc.).

The macro generates a module `localisation` with a few things:
- an enum for your locales;
- a const array of all available locales (for e.g. iteration);
- a function for every key in your file, to the text;
- a static `RwLock` to keep the currently set locale.

### Locale choice
The user does not need to hold onto any state, since the locale setting is kept in a static `RwLock`, accessible directly (`localisation::LOCALE`) or through `localisation::set_locale()` and `localisation::get_locale()`. During the `load!` macro, it is set to the first declared locale.

Using an `RwLock` means that you can have concurrent reads of the locale, and only block for writes. This makes sense -- you wouldn't want to change the locale very often, I imagine. 

`Locale` implements `std::fmt::Display`, and will print the name provided in parentheses or the basic key if no name is provided.

### File structure
The file structure is designed to attempt to find a balance between ease-of-use and ease-of-parsing (which affects compile time). A minimal example:
```toml
# Declare english and italian as locales
!locales en it

# Define an entry with key 'greet'
greet:
    # Define english text
    en "Hi!"
    # Define italian text
    it "Ciao!"

```
#### Comments
Comments are parsed as anything starting with a `#` and ending with a newline. 
There are two exceptions: 
1) comments may not be inserted between a locale and its value; and
2) comments are not parsed inside values.

#### Config
A config line is a `!` followed y a key and one or more values, all on the same line. 

These are the currently available config keys:
- `!locales` is used to declare locales, separated by whitespace. This must occur before any text entries using them.

- `!include` appends one or more files' contents to be parsed, in the order read.

- `!scope` declares everything following it to be in a scope. This translates directly to a structure of `pub mod`s, in a pretty straightforward way. A file included after `!scope a` is genereated completely inside `pub mod a { .. }`. Note that declaring several scopes in one file will *not* nest them.

#### Entries
The rest of the file must contain entries, each is a key followed by a colon `:` and at least one pair of a locale and a quote-enclosed value. 

Keys and locales must both start with an ASCII alphabetical character and only contain ASCII alphanumerics, hyphens `-`, and underscores `_`, but are case-insensitve (`_` is considered to be the lowercase version of `-`).

#### Values and formatting
A value may contain any valid UTF-8. Quotes and curly braces may be escaped with a backslash `\`. The strings are passed wholesale to `format!`, and so any regular formatting will work, e.g. `"Hello {name}, I'm {dist:.2} light-years away."`.

> ***Note***
>
> You may use unnamed parameters like `{0}` or `{}`, but as they need proper names to be passed into functions, they will be renamed to `arg0` etc. This means that using both `{0}` and `arg0` will create overlap. I don't foresee this being a problem for anyone, though.

## Accessing text
The `text!` macro is designed to fit in as a replacement for `format!`, where the string literal is replaced by a key from the loaded file. It matches on the locale to choose which localised text to format, inserting arguments as `format!` would.

When writing the texts in the file, arguments may be entered just like for `format!`. 

### Example:
`strings.txt`
```toml
!locales en
text:
    en "Hello!"

text-with-args: 
    en "Hi {name}, I'm {me}."
```
rust-side:
```rust
use safflower::{text, load};

load!("strings.txt");

let foo = "foo";
let bar = 42;

assert_eq!(
    text!(text),
    format!("Hello!"),
);

assert_eq!(
    text!(text_with_args, foo, bar),
    format!("Hi {foo}, I'm {bar}."),
);
```
