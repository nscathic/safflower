# `safflower` 
*Statically-Allocated Fromat-Friendly Localising (Written Entirely in Rust)*

[![Crates.io Version](https://img.shields.io/crates/v/safflower)](https://crates.io/crates/safflower)
[![docs](https://img.shields.io/docsrs/safflower?logo=rust)](https://docs.rs/safflower/latest/)
[![Repo](https://img.shields.io/badge/github-repo-blue?logo=github)]()
[![License](https://img.shields.io/badge/license-MIT-blue)](#license)


`safflower` aims to provide a no-fuss text localiser with minimum runtime overhead.
It does so in two main ways: 
1) all the text is read, checked, and parsed at compile time; and
2) all localisations of the text are available at the same time.

Below is a description of how it works. If you just want to see it in use, skip to [Accessing text](#accessing-text).

## Loading
The `load!` macro reads a text file by the path provided. If there are any errors in the file (mainly from formatting), they are caught at compile time. This removes the need for runtime error handling (path exists, file can be read, contents can be parsed, key exists, etc.).

The macro generates a module `localisation` with a few things:
- an enum for your locales;
- a const array of all available locales (for e.g. iteration);
- a function for every key in your file, to the text;
- a static `Mutex` to keep the currently set locale.

### Locale choide
The locale being kept in a `Mutex` means that the user doesn't need to hold onto any state -- it is globally available. 

It can be set with `localisation::set_locale()` and gotten with `localisation::get_locale()`. During the `load!` macro, it is set to the first declared locale.

I'm not a fan of global variables, but I think this makes sense here: we don't expect it to change a lot, maybe not at all during the lifetime of the program, but every single piece of text depends on it. This also means that the text getting functions are *thread blocking* -- if someone has a better solution, feel free to say so.

>*Note*
>
>In a small benchmark on an old laptop, one million calls to `text!` for a 256-byte string took about ~230 ms, whereas one million calls to `format!` for the same text took about ~120 ms. 

### File structure
The file structure is designed to attempt to find a balance between ease-of-use and ease-of-parsing (which affects compile time). 

Comments are parsed as anything starting with a `#` and ending with a newline. 
There are two exceptions: 
1) comments may not be inserted between a locale and its value; and
2) comments are not parsed inside values.

The file may start with a number of config lines, each starting with `!`.

*As it currently stands, the file must start with a config line stating the used locales.* 

The rest of the file must contain entries, each with a key followed by a colon `:` and at least one locale. Each locale must be followed by a value enclosed in quotes `"`. 

Keys and locales must both only contain ASCII alphanumerics, hyphen `-`, and underscore `_`, but are case-insensitve (`_` is considered to be the lowercase version of `-`). *Note that a value may contain any valid UTF-8.*

A minimal example:
```t
# Declare english and italian as locales
!locales en it

# Define an entry with key 'greet'
greet:
    # Define english text
    en "Hi!"
    # Define italian text
    it "Ciao!"

```

## Accessing text
The `text!` macro is designed to fit in as a replacement for `format!`, where the string literal is replaced by a key from the loaded file. It matches on the locale to choose which localised text to format, inserting arguments as `format!` would.

When writing the texts in the file, arguments may be entered just like for `format!`. 

### Example:
`strings.txt`
```t
!locales en
text:
    en "Hello!"

text-with-args: 
    en "Hi {name}, I'm {me}."
```
rust-side:
```rust
# use safflower::{text, load};
load!("strings.txt");

let foo = "foo";
let bar = "bar";

assert_eq!(
    text!(text),
    format!("Hello!"),
);

assert_eq!(
    text!(text_with_args, foo, bar),
    format!("Hi {foo}, I'm {bar}."),
);
```
