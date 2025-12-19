# `safflower` 
*Statically-Allocated Fromat-Friendly Localising (Written Entirely in Rust)*

`safflower` aims to provide a no-fuss text localiser with minimum runtime overhead.
It does so in two main ways: 
1) all the text is read, checked, and parsed at compile time; and
2) the localised text is available via the `text!` macro, automatically picking the correct version by reading an environment variable.

## Loading
The `load!` macro reads a text file by the path provided. If there are any errors in the file (mainly from formatting), they are caught at compile time. This removes the runtime need for error handling (path exists, file can be read, contents can be parsed, etc.).

Successful loading and parsing creates constants in the same scope as the one in which the macro was called. 

### File structure
The file structure is designed to attempt to find a balance between ease-of-use and ease-of-parsing (which affects compile time). 

Comments are parsed as anything starting with a `#` and ending with a newline. 
There are two exceptions: 
1) comments may not be inserted between a locale and its value; and
2) comments may not occur inside values.

The file may start with a number of config lines, each starting with `!`.

*As it currently stands, the file must start with a config line stating the used locales.* 

The rest of the file must contain entries, each with a key followed by a colon `:` and at least one locale. Each locale must be followed by a value enclosed in quotes `"`. 

Keys and locales must both only contain ASCII alphanumerics, hyphen `-`, and underscore `_`, but are case-insensitve (`_` is considered to be the lowercase version of `-`). *Note that a value may contain any valid UTF-8.*

A minimal example:
```
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
The `text!` macro is designed to fit in as a replacement for `format!`, where the string literal is replaced by a key from the loaded file. It matches on an environment variable to choose which localised text to format, inserting arguments as `format!` would. If the envrionment variable is not set, can't be read, or does not equal any of the expected values, the first locale declared is used by default.

When writing the texts in the file, arguments may be entered just like for `format!`. 

### Example:
`file.txt`
```
!locales en
text-with-args: 
    en "Hi {name}, I'm {me}."
```
rust-side:
```rust
let foo = "foo";
let bar = "bar";

load!("file.txt");
assert_eq!(
    text!(text_with_args, foo, bar),
    format!("Hi {foo}, I'm {bar}."),
);
```

