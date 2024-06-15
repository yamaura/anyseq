use crate::de::{self, Deserializer};
use crate::{Lexer, Token};

pub struct TokenIter;

impl<I, O> Lexer<I, Token<O>> for TokenIter
where
    I: Iterator<Item = Token<O>>,
{
    type Error = ();

    fn item(input: &mut I) -> Result<Token<O>, Self::Error> {
        match input.next() {
            Some(item) => Ok(item),
            None => Ok(Token::Eof),
        }
    }
}

pub fn from_iter<'de, T, I>(
    iter: I,
) -> Result<T, de::Error<<TokenIter as Lexer<I, Token<&'de str>>>::Error>>
where
    I: Iterator<Item = Token<&'de str>>,
    T: serde::Deserialize<'de>,
{
    let mut de = Deserializer::<TokenIter, _, _>::new(iter);
    T::deserialize(&mut de)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[test]
    fn test_seq_access() {
        let tokens = vec![
            Token::SeqStart { len: None },
            Token::Item("1"),
            Token::Item("2"),
            Token::Item("3"),
            Token::SeqEnd,
        ];
        assert_eq!(
            from_iter::<Vec<i8>, _>(tokens.into_iter()).unwrap(),
            vec![1, 2, 3]
        );
    }

    #[test]
    fn test_struct() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Test {
            name: String,
            age: u32,
        }

        let tokens = vec![
            Token::MapStart { len: None },
            Token::Item("age"),
            Token::Item("21"),
            Token::Item("name"),
            Token::Item("John"),
            Token::MapEnd,
        ];
        assert_eq!(
            from_iter::<Test, _>(tokens.into_iter()).unwrap(),
            Test {
                name: "John".to_string(),
                age: 21
            }
        );

        let tokens = vec![
            Token::SeqStart { len: None },
            Token::MapStart { len: None },
            Token::Item("name"),
            Token::Item("John"),
            Token::Item("age"),
            Token::Item("21"),
            Token::MapEnd,
            Token::MapStart { len: None },
            Token::Item("name"),
            Token::Item("Jane"),
            Token::Item("age"),
            Token::Item("31"),
            Token::MapEnd,
            Token::SeqEnd,
        ];

        assert_eq!(
            from_iter::<Vec<Test>, _>(tokens.into_iter()).unwrap(),
            vec![
                Test {
                    name: "John".to_string(),
                    age: 21
                },
                Test {
                    name: "Jane".to_string(),
                    age: 31
                }
            ]
        );
    }
}
