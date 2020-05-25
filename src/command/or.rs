use super::{Parser, ParserBase, Input};

#[derive(Clone, Copy, Debug)]
pub struct Or<T, U> {
    pub(super) first: T,
    pub(super) second: U,
}

impl<T, U> ParserBase for Or<T, U>
where
    T: Parser,
    U: Parser<Extract = T::Extract>,
{
    type Extract = T::Extract;

    fn parse<'i>(
        &self,
        input: &mut Input<'i>,
    ) -> Option<Self::Extract> {
        match self.first.parse(&mut input.clone()) {
            ok @ Some(_) => ok,
            _ => self.second.parse(input),
        }
    }
}
