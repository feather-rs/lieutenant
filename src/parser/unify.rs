use super::{Either, Input, Parser, ParserBase, Tuple};

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

    fn parse<'i>(&self, input: &mut Input<'i>) -> Option<Self::Extract> {
        let (ex,) = self.parser.parse(input)?;
        match ex {
            Either::A(a) => Some(a),
            Either::B(b) => Some(b),
        }
    }
}
