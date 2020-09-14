use super::{Input, Parser, ParserBase, Tuple, Result};

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
    fn parse<'i>(&self, input: &mut Input<'i>) -> Result<Self::Extract> {
        match self.parser.parse(input) {
            Ok((arg,)) => Ok(arg),
            Err(err) => Err(err),
        }
    }
}
