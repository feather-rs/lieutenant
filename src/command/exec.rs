use super::{Input, Parser, ParserBase};

#[derive(Clone)]
pub struct Exec<P: Parser, C> {
    pub(super) parser: P,
    pub(super) command: for<'a> fn(&'a mut C, &'a P::Extract) -> ()
}

impl<P, C> ParserBase for Exec<P, C>
where
    P: Parser,
    P::Extract: 'static,
{
    type Extract = (ParsedCommand<P::Extract, C>,);

    fn parse<'i>(&self, input: &mut Input<'i>) -> Option<Self::Extract> {
        let ex = self.parser.parse(input)?;
        if input.is_empty() {
            Some((ParsedCommand {
                extracted: ex,
                command: self.command,
            },))
        } else {
            None
        }
    }
}

pub struct ParsedCommand<E, C> {
    pub(super) extracted: E,
    pub(super) command: for<'a> fn(&'a mut C, &'a E) -> ()
}

pub trait Command<C> {
    fn call(&self, ctx: &mut C);
}

impl<E, C> Command<C> for ParsedCommand<E, C> {
    fn call(&self, ctx: &mut C) {
        let command = self.command;
        command(ctx, &self.extracted)
    }
}