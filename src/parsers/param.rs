use crate::parser::{ParserBase, Result, Error};
use crate::Input;

#[derive(Debug, Clone)]
pub struct Param<T> {
    param: std::marker::PhantomData<T>,
}

impl<T> ParserBase for Param<T>
where
    T: std::str::FromStr,
{
    type Extract = (T,);

    fn parse<'i>(&self, input: &mut Input<'i>) -> Result<Self::Extract> {
        let head = input.advance_until(" ");
        match T::from_str(head) {
            Ok(ok) => Ok((ok,)),
            Err(_) => Err(Error::Todo),
        }
    }
}

pub fn param<T: std::str::FromStr>() -> Param<T> {
    Param {
        param: Default::default(),
    }
}