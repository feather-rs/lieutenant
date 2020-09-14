use crate::parser::{ParserBase, Result};
use crate::Input;

#[derive(Debug, Clone)]
pub struct Any;

impl ParserBase for Any {
    type Extract = ();

    #[inline]
    fn parse<'i>(&self, _input: &mut Input<'i>) -> Result<Self::Extract> {
        Ok(())
    }
}

pub fn any() -> Any {
    Any
}