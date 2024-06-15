use super::{Lexer, Token};
use serde::de::{DeserializeSeed, Visitor};
use std::marker::PhantomData;

macro_rules! deserialize_fn_inner {
    ($name:ident, $ty:ident, $parse:expr) => {
        paste::paste! {
            fn [<deserialize_ $name>]<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: Visitor<'de>,
            {
                use std::str::FromStr;
                self.parse_item().map(|t| match t {
                    Token::Item(s) => visitor.[<visit_ $name>]($parse(s)?),
                    other => Err(Error::InvalidToken(format!("{:?}", other)))?,
                }).map_err(Error::LexerError)?
            }
        }
    };
}

macro_rules! deserialize_fn {
    ($ty:ident) => {
        deserialize_fn_inner!($ty, $ty, <$ty>::from_str);
    };
}

#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[error("{0}")]
pub enum Error<E>
where
    E: std::fmt::Debug,
{
    ParseIntError(#[from] std::num::ParseIntError),
    ParseFloatError(#[from] std::num::ParseFloatError),
    ParseBoolError(#[from] std::str::ParseBoolError),
    DeError(#[from] serde::de::value::Error),
    #[error("LexerError({0:?})")]
    LexerError(E),
    #[error("InvalidToken({0})")]
    InvalidToken(String),
}

impl<E> serde::de::Error for Error<E>
where
    E: std::fmt::Debug,
{
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::DeError(serde::de::value::Error::custom(msg))
    }
}

pub struct Deserializer<L, I, O> {
    pub(crate) input: I,
    pub(crate) peeked: Option<O>,
    _marker: PhantomData<fn() -> L>, //    pub(crate) lexer: &'de L
}

impl<I, L, O> Deserializer<L, I, O> {
    pub fn new(input: I) -> Deserializer<L, I, O> {
        Deserializer {
            input,
            peeked: None,
            _marker: PhantomData,
        }
    }
}

impl<I, L, O> Deserializer<L, I, O>
where
    L: Lexer<I, O>,
{
    pub(crate) fn parse_item(&mut self) -> Result<O, L::Error> {
        if let Some(token) = self.peeked.take() {
            Ok(token)
        } else {
            L::item(&mut self.input)
        }
    }

    pub(crate) fn peek_item(&mut self) -> Result<&O, L::Error> {
        if self.peeked.is_none() {
            self.peeked = Some(self.parse_item()?)
        }
        match self.peeked {
            Some(ref token) => Ok(token),
            None => unreachable!(),
        }
    }
}

impl<'de, 'a, I, L: Lexer<I, Token<&'de str>>> serde::de::Deserializer<'de>
    for &'a mut Deserializer<L, I, Token<&'de str>>
where
    L::Error: std::fmt::Debug,
{
    //type Error = serde::de::value::Error;
    type Error = Error<L::Error>;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    deserialize_fn!(bool);
    deserialize_fn!(i8);
    deserialize_fn!(i16);
    deserialize_fn!(i32);
    deserialize_fn!(i64);
    deserialize_fn!(i128);
    deserialize_fn!(u8);
    deserialize_fn!(u16);
    deserialize_fn!(u32);
    deserialize_fn!(u64);
    deserialize_fn!(u128);
    deserialize_fn!(f32);
    deserialize_fn!(f64);

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_item()
            .map(|t| match t {
                Token::Item(s) => visitor.visit_borrowed_str(s),
                other => Err(Error::InvalidToken(format!("{:?}", other)))?,
            })
            .map_err(Error::LexerError)?
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_item()
            .map(|t| match t {
                Token::Item(s) => visitor.visit_string(s.to_string()),
                other => Err(Error::InvalidToken(format!("{:?}", other)))?,
            })
            .map_err(Error::LexerError)?
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let item = self.peek_item().map_err(Error::LexerError)?;
        if matches!(item, Token::MapStart { .. }) {
            self.parse_item().map_err(Error::LexerError)?; // consume the MapStart
        }
        visitor.visit_map(&mut MapAccess { de: self })
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        use Token::*;
        let item = self.peek_item().map_err(Error::LexerError)?;
        match item {
            Item(_) | SeqStart { .. } => self.deserialize_seq(visitor),
            MapStart { .. } => self.deserialize_map(visitor),
            SeqEnd | MapEnd | Eof => Err(Error::InvalidToken(format!(
                "{:?} at deserialize_struct",
                item
            )))?,
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let item = self.peek_item().map_err(Error::LexerError)?;
        if matches!(item, Token::SeqStart { .. }) {
            self.parse_item().map_err(Error::LexerError)?;
        }
        visitor.visit_seq(&mut SeqAccess { de: self })
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    serde::forward_to_deserialize_any! {
        /*bool i32 i64 i128 u8 u16 u32 u64 u128 f32 f64*/ char /*str string*/
        bytes byte_buf /*option*/ unit unit_struct newtype_struct /*tuple*/
        tuple_struct enum ignored_any
    }
}

struct SeqAccess<'a, L, I, O> {
    de: &'a mut Deserializer<L, I, O>,
}

impl<'de, 'a, I, L: Lexer<I, Token<&'de str>>> serde::de::SeqAccess<'de>
    for &'a mut SeqAccess<'a, L, I, Token<&'de str>>
where
    L::Error: std::fmt::Debug,
{
    type Error = Error<L::Error>;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        use Token::*;
        self.de
            .peek_item()
            .map(|t| matches!(t, SeqEnd | Eof))
            .map(|t| match t {
                true => {
                    let _ = self.de.parse_item(); // consume the SeqEnd
                    Ok(None)
                }
                false => seed.deserialize(&mut *self.de).map(Some),
            })
            .map_err(Error::LexerError)?
    }
}

struct MapAccess<'a, L, I, O> {
    de: &'a mut Deserializer<L, I, O>,
}

impl<'de, 'a, I, L: Lexer<I, Token<&'de str>>> serde::de::MapAccess<'de>
    for &'a mut MapAccess<'a, L, I, Token<&'de str>>
where
    L::Error: std::fmt::Debug,
{
    type Error = Error<L::Error>;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        let item = self.de.peek_item().map_err(Error::LexerError)?;
        match item {
            Token::Item(_) => seed.deserialize(&mut *self.de).map(Some),
            Token::MapEnd => {
                let _ = self.de.parse_item(); // consume the MapEnd
                Ok(None)
            }
            _ => Err(Error::InvalidToken(format!("{:?}", item))),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let item = self.de.peek_item().map_err(Error::LexerError)?;
        match item {
            Token::Item(_) => seed.deserialize(&mut *self.de),
            _ => Err(Error::InvalidToken(format!("{:?}", item))),
        }
    }
}

pub struct DeserializerIter<L, I, O, T> {
    de: Deserializer<L, I, O>,
    _marker: PhantomData<T>,
}

impl<'de, I, L, T> core::iter::Iterator for DeserializerIter<L, I, Token<&'de str>, T>
where
    L: Lexer<I, Token<&'de str>>,
    L::Error: std::fmt::Debug,
    T: serde::Deserialize<'de>,
{
    type Item = Result<T, Error<L::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        let t = self.de.peek_item();
        match t {
            Ok(Token::Eof) => None,
            Ok(_) => Some(serde::de::Deserialize::deserialize(&mut self.de)),
            Err(e) => Some(Err(Error::LexerError(e))),
        }
    }
}
