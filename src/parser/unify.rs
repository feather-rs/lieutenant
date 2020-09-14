use super::{Either, Input, Parser, ParserBase, Tuple, Result};

#[derive(Clone, Copy, Debug)]
pub struct Unify<F> {
    pub(super) parser: F,
}

impl<F, T> ParserBase for Unify<F>
where
    F: Parser<Extract = (Either<T, T>,)>,
    T: Tuple,
{
    type Extract = T;

    #[inline]
    fn parse<'i>(&self, input: &mut Input<'i>) -> Result<Self::Extract> {
        let (ex,) = self.parser.parse(input)?;
        Ok(match ex {
            Either::A(a) => a,
            Either::B(b) => b,
        })
    }
}
