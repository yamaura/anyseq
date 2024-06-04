use winnow::{
    ascii::line_ending,
    combinator::{alt, eof, opt, preceded},
    stream::{Compare, Stream, StreamIsPartial},
    token::take_while,
    PResult, Parser,
};

use crate::de::{self, Deserializer};
use crate::{Lexer, Token};

pub struct Csv;

impl<I, O> Lexer<I, Token<O>> for Csv
where
    I: StreamIsPartial + Stream<Token = char, Slice = O> + Compare<char> + Compare<&'static str>,
{
    type Error = winnow::error::ErrMode<winnow::error::ContextError>;

    fn item(input: &mut I) -> Result<Token<I::Slice>, Self::Error> {
        alt((
            eof.map(|_| Token::SeqEnd),
            line_ending.map(|_| Token::SeqEnd),
            preceded(opt(','), take_while(0.., |c| c != ',')).map(Token::Item),
        ))
        .parse_next(input)
    }
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T, de::Error<<Csv as Lexer<&str, Token<&str>>>::Error>>
where
    T: serde::de::Deserialize<'a>,
{
    let mut deserializer = Deserializer::<Csv, _, _>::new(s);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        panic!("input not empty")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv() {
        assert_eq!(
            Csv::item.parse_peek("a,b,c").unwrap(),
            (",b,c", Token::Item("a"))
        );
        assert_eq!(
            Csv::item.parse_peek("").unwrap(),
            ("", Token::<&str>::SeqEnd)
        );
        assert_eq!(vec![1, 2, 3], from_str::<Vec<i8>>("1,2,3").unwrap());
        assert_eq!(
            vec!["a", "b", "3"],
            from_str::<Vec<String>>("a,b,3").unwrap()
        );
    }
}
