use super::{Input, Parser, ParserBase, Tuple};

#[derive(Clone, Copy, Debug)]
pub struct UntupleOne<P> {
    pub(super) parser: P,
}

impl<P, T> ParserBase for UntupleOne<P>
where
    P: Parser<Extract = (T,)>,
    T: Tuple,
{
    type Extract = T;

    #[inline]
    fn parse<'i>(&self, input: &mut Input<'i>) -> Option<Self::Extract> {
        match self.parser.parse(input) {
            Some((arg,)) => Some(arg),
            None => None,
        }
    }
}
