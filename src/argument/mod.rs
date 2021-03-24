mod numbers;
use crate::parser::parser::IterParser;
pub use numbers::*;

pub trait Argument {
    type Parser: IterParser<Extract = (Self,), ParserState = Self::ParserState> + Sized + Default;
    type ParserState: Default;
}
