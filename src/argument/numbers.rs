use crate::parser::IterParser;

use super::Argument;

pub struct U32Parser {}

impl IterParser for U32Parser {
    type Extract = (u32,);

    type ParserState = ();

    fn parse<'p>(
        &self,
        _state: Self::ParserState,
        input: &'p str,
    ) -> (
        anyhow::Result<(Self::Extract, &'p str)>,
        Option<Self::ParserState>,
    ) {
        // Consume digit from head of input

        let mut iter = input.char_indices();
        let mut index = 0;
        if let Some((i, c)) = iter.next() {
            if c == '+' || c == '-' {
                index = i;
            }
        } else {
            return (Err(anyhow::anyhow!("Empty input")), None);
        }

        for (i, c) in iter {
            if !c.is_digit(10) {
                break;
            }
            index = i
        }

        match input[0..=index].parse::<u32>() {
            Ok(number) => return (Ok(((number,), input)), None),
            Err(_) => return (Err(anyhow::anyhow!("Not a number")), None),
        };
    }

    fn regex(&self) -> String {
        "[\\+|-]?\\d+".into()
    }
}

impl Default for U32Parser {
    fn default() -> Self {
        Self {}
    }
}

impl Argument for u32 {
    type Parser = U32Parser;
    type ParserState = ();
}
