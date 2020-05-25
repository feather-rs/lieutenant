use super::{Func, Parser, ParserBase, Input};
#[derive(Clone)]
pub struct Map<P, F> {
    pub(super) parser: P,
    pub(super) callback: F,
}

impl<P, F> ParserBase for Map<P, F>
where
    P: Parser,
    F: Func<P::Extract> + Clone,
{
    type Extract = (F::Output,);

    #[inline]
    fn parse<'i>(&self, input: &mut Input<'i>) -> Option<Self::Extract> {
        let ex = self.parser.parse(input)?;
        Some((self.callback.call(ex),))
    }
}