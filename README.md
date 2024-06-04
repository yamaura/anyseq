# anyseq

anyseq is a Rust crate that provides a trait named `Lexer` for splitting structured text into string tokens.
By implementing this trait, `Deserializer` for `serde` are automatically generated.

## Features

- Provides a trait named `Lexer` for creating token steam.
- Automatically generates `serde` Deserializer when the trait is implemented.
- Simplifies the implementation of Deserializer, such as lexers that split comma-separated strings into tokens.
- `Lexer` is compatibility with the `winnow` crate, making implementation easier when using `winnow`.

## Usage

```rust
use anyseq::{Token, Lexer};
use anyseq::de::Deserializer;
use serde::Deserialize;

struct IterLexer;

impl<I, O> Lexer<I, Token<O>> for IterLexer 
where
I: Iterator<Item = O>
{
    type Error = Box<dyn std::error::Error>;
    fn item(input: &mut I) -> Result<Token<O>, Self::Error> {
        Ok(match input.next() {
                Some(t) => Token::Item(t),
                None => Token::SeqEnd,
        })
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct Test {
    a: i32,
    b: String,
}

let mut d = Deserializer::<IterLexer, _, _>::new(["42", "foo"].into_iter());
assert_eq!(
    Test {
        a: 42,
        b: "foo".to_string(),
    },
    Test::deserialize(&mut d).unwrap()
);
```
