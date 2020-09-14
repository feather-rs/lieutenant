
use super::{Parser, ParserBase, Tuple, HList, Combine, Input, Func, Result, Lazy};
use crate::command::CommandMapping;

pub struct Executor<P, F> {
    pub(super) parser: P,
    pub(super) callback: F,
}

impl<P, F> ParserBase for Executor<P, F>
where
    P: Parser,
    P::Extract: Lazy,
    F: Func<<P::Extract as Lazy>::Output> + Clone,
{
    type Extract = (CommandMapping<P::Extract, F>,);

    #[inline]
    fn parse<'i>(&self, input: &mut Input<'i>) -> Result<Self::Extract> {
        let arguments = self.parser.parse(input)?;
        Ok((CommandMapping::new(arguments, self.callback.clone()),))
    }
}