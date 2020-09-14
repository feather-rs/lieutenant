use super::{Either, Input, Parser, ParserBase, Result};

#[derive(Clone, Copy, Debug)]
pub struct Alt<T, U> {
    pub(super) first: T,
    pub(super) second: U,
}

impl<T, U> ParserBase for Alt<T, U>
where
    T: Parser,
    U: Parser,
{
    type Extract = (Either<T::Extract, U::Extract>,);

    #[inline]
    fn parse<'i>(&self, input: &mut Input<'i>) -> Result<Self::Extract> {
        self.first
            .parse(&mut input.clone())
            .map(Either::A)
            .or_else(|_| self.second.parse(input).map(Either::B))
            .map(|e| (e,))
    }
}
