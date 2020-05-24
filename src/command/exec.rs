use super::{Input, Parser, ParserBase};

#[derive(Clone)]
pub struct Exec<'a, P: Parser, C> {
    pub(super) parser: P,
    pub(super) command: fn(&'a mut C, &'a P::Extract) -> ()
}

impl<'a, P, C> ParserBase for Exec<'a, P, C>
where
    P: Parser,
    C: 'a,
    P::Extract: 'a,
{
    type Extract = (Command<'a, P::Extract, C>,);

    fn parse<'i>(&self, input: &mut Input<'i>) -> Option<Self::Extract> {
        let ex = self.parser.parse(input)?;
        if input.is_empty() {
            Some((Command {
                extracted: ex,
                command: self.command,
            },))
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Command<'a, E, C> {
    pub(super) extracted: E,
    pub(super) command: fn(&'a mut C, &'a E) -> ()
}

impl<'a, E, C> Command<'a, E, C> {
    pub fn call(&'a self, ctx: &'a mut C) -> () {
        let command = self.command;
        command(ctx, &self.extracted)
    }
}