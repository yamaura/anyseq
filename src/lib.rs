#![doc = include_str!("../README.md")]

#[cfg(feature = "csv")]
pub mod csv;
pub mod de;
pub mod iter;
#[cfg(feature = "spaces")]
pub mod spaces;

#[derive(Debug, PartialEq, Clone)]
pub enum Token<T> {
    Item(T),
    SeqStart { len: Option<usize> },
    SeqEnd,
    MapStart { len: Option<usize> },
    MapEnd,
    Eof,
}

pub trait Lexer<I, O> {
    type Error;

    fn item(input: &mut I) -> Result<O, Self::Error>;
}
