use super::{Combine, Parser, ParserBase, HList, Input, Tuple};

#[derive(Debug, Copy, Clone)]
pub struct And<T, U> {
    pub(super) first: T,
    pub(super) second: U,
}

impl<T, U> ParserBase for And<T, U>
where
    T: Parser,
    U: Parser,
    <T::Extract as Tuple>::HList: Combine<<U::Extract as Tuple>::HList> + Send,
{
    type Extract = <<<T::Extract as Tuple>::HList as Combine<<U::Extract as Tuple>::HList>>::Output as HList>::Tuple;

    fn parse<'i>(
        &self,
        input: &mut Input<'i>,
    ) -> Option<Self::Extract> {
        let first = self.first.parse(input)?.hlist();
        let second = self.second.parse(input)?.hlist();
        Some(first.combine(second).flatten())
    }
}
