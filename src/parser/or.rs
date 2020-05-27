use super::{Either, Input, Parser, ParserBase};

#[derive(Clone, Copy, Debug)]
pub struct Or<T, U> {
    pub(super) first: T,
    pub(super) second: U,
}

impl<T, U> ParserBase for Or<T, U>
where
    T: Parser,
    U: Parser,
{
    type Extract = (Either<T::Extract, U::Extract>,);

    #[inline]
    fn parse<'i>(&self, input: &mut Input<'i>) -> Option<Self::Extract> {
        self.first
            .parse(&mut input.clone())
            .map(Either::A)
            .or_else(|| self.second.parse(input).map(Either::B))
            .map(|e| (e,))
    }
}
