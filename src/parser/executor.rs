
use std::marker::PhantomData;
use super::{Parser, ParserBase, Tuple, HList, Combine, Input, Func, Result};
use crate::command::CommandMapping;

pub struct Executor<P, S, F> {
    pub(super) parser: P,
    pub(super) state: PhantomData<S>,
    pub(super) callback: F,
}

impl<P, S, F> ParserBase for Executor<P, S, F>
where
    P: Parser,
    <P::Extract as Tuple>::HList: Combine<S::HList>,
    S: Tuple,
    F: Func<<<<P::Extract as Tuple>::HList as Combine<S::HList>>::Output as HList>::Tuple> + Clone,
{
    type Extract = (CommandMapping<P::Extract, S, F>,);

    #[inline]
    fn parse<'i>(&self, input: &mut Input<'i>) -> Result<Self::Extract> {
        let arguments = self.parser.parse(input)?;
        Ok((CommandMapping::new(arguments, self.callback.clone()),))
    }
}