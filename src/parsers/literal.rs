use crate::parser::{Error, ParserBase, Result};
use crate::Input;
use unicase::UniCase;
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct Literal {
    value: Cow<'static, str>,
    unicase: UniCase<Cow<'static, str>>,
}

impl ParserBase for Literal
{
    type Extract = ();

    fn parse<'i>(&self, input: &mut Input<'i>) -> Result<Self::Extract> {
        let head = input
            .take_bytes(self.value.len())
            // TODO remove clone
            .ok_or(Error::Literal(self.value.clone()))?;
        input.trim_start();
        let head_unicase = UniCase::new(head);
        if self.unicase == head_unicase {
            Ok(())
        } else {
            Err(Error::Literal(self.value.clone()))
        }
    }
}

pub fn literal<L>(lit: L) -> Literal
where
    L: Into<Cow<'static, str>>
{
    let lit = lit.into();
    assert!(!lit.is_empty());
    assert!(lit.chars().all(|c| c != ' '));
    // TODO remove clone
    Literal {
        value: lit.clone(),
        unicase: UniCase::new(lit),
    }
}
