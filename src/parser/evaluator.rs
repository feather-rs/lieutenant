#[cfg(test)]
use super::IterParser;

/*
This file contains  convenience struct that i use for testing.
*/

#[cfg(test)]
pub(crate) struct Evaluator<'p, P> {
    parser: &'p P,
}
#[cfg(test)]
impl<'p, P: IterParser> Evaluator<'p, P> {
    pub(crate) fn new(parser: &'p P) -> Self {
        Self { parser }
    }
}
#[cfg(test)]
impl<'p, P: IterParser> Evaluator<'p, P> {
    pub(crate) fn evaluate_all<'i>(
        &self,
        input: &'i str,
    ) -> Vec<anyhow::Result<(P::Extract, &'i str)>> {
        let mut result = Vec::new();

        let mut state = Some(P::ParserState::default());

        while let Some(st) = state {
            let (res, new_st) = self.parser.parse(st, input);
            result.push(res);
            state = new_st;
        }

        return result;
    }
}
