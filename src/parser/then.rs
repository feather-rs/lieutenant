use super::{Combine, HList, Input, Parser, ParserBase, Tuple, Result};

type Combined<T, U> = <<<<T as ParserBase>::Extract as Tuple>::HList as Combine<
    <<U as ParserBase>::Extract as Tuple>::HList,
>>::Output as HList>::Tuple;

#[derive(Debug, Copy, Clone)]
pub struct Then<T, U> {
    pub(super) first: T,
    pub(super) second: U,
}

impl<T, U> ParserBase for Then<T, U>
where
    T: Parser,
    U: Parser,
    <T::Extract as Tuple>::HList: Combine<<U::Extract as Tuple>::HList>,
{
    type Extract = Combined<T, U>;
    
    #[inline]
    fn parse<'i>(&self, input: &mut Input<'i>) -> Result<Self::Extract> {
        let first = self.first.parse(input)?.hlist();
        let second = self.second.parse(input)?.hlist();
        Ok(first.combine(second).flatten())
    }
}
