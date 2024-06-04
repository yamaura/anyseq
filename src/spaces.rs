use winnow::{
    ascii::{line_ending, space0},
    combinator::{alt, eof, preceded},
    stream::{Compare, Stream, StreamIsPartial},
    token::take_while,
    PResult, Parser,
};

use crate::de::{self, Deserializer};
use crate::{Lexer, Token};

pub struct Spaces;

impl<I, O> Lexer<I, Token<O>> for Spaces
where
    I: StreamIsPartial + Stream<Token = char, Slice = O> + Compare<char> + Compare<&'static str>,
{
    type Error = winnow::error::ErrMode<winnow::error::ContextError>;

    fn item(input: &mut I) -> Result<Token<I::Slice>, Self::Error> {
        alt((
            eof.map(|_| Token::SeqEnd),
            line_ending.map(|_| Token::SeqEnd),
            preceded(
                space0,
                take_while(0.., |c| c != ' ' && c != '\t' && c != '\n' && c != '\r'),
            )
            .map(Token::Item),
        ))
        .parse_next(input)
    }
}

pub fn from_str<'a, T>(
    s: &'a str,
) -> Result<T, de::Error<<Spaces as Lexer<&str, Token<&str>>>::Error>>
where
    T: serde::de::Deserialize<'a>,
{
    use serde::de::Error;

    let mut deserializer = Deserializer::<Spaces, _, _>::new(s);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(de::Error::custom(format!(
            "input not empty: {:?}",
            deserializer.input
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_space0() {
        assert_eq!(vec![1, 2, 3], from_str::<Vec<i8>>("1 2 3").unwrap());

        #[derive(Debug, PartialEq, serde::Deserialize)]
        struct Struct<'a> {
            a: i8,
            b: Option<&'a str>,
            #[serde(default)]
            c: Option<u32>,
        }
        assert_eq!(
            Struct {
                a: 1,
                b: Some("foo"),
                c: None
            },
            from_str::<Struct>("1 foo").unwrap()
        );
        assert_eq!(
            Struct {
                a: 1,
                b: Some("foo"),
                c: Some(5)
            },
            from_str::<Struct>("1 foo 5").unwrap()
        );
    }
}
