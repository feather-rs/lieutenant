use super::{Input, Func, Tuple, Combine, HList, Either, Parser, ParserBase};

#[derive(Clone)]
pub struct Exec<P, F> {
    pub(super) parser: P,
    pub(super) command: F,
}

impl<P, F> ParserBase for Exec<P, F>
where
    P: Parser,
    P::Extract: 'static,
    F: Clone,
{
    type Extract = (ParsedCommand<P::Extract, F>,);

    fn parse<'i>(&self, input: &mut Input<'i>) -> Option<Self::Extract> {
        let ex = self.parser.parse(input)?;
        if input.is_empty() {
            Some((ParsedCommand {
                extracted: ex,
                command: self.command.clone(),
            },))
        } else {
            None
        }
    }
}

pub struct ParsedCommand<E, F> {
    pub(super) extracted: E,
    pub(super) command: F
}

pub trait Command<C> {
    fn call(self, ctx: C);
}

impl<A, B, C> Command<C> for Either<(A,), (B,)>
where
    A: Command<C>,
    B: Command<C>,
{
    fn call(self, ctx: C) {
        match self {
            Either::A((a,)) => a.call(ctx),
            Either::B((b,)) => b.call(ctx),
        }
    }
}

impl<E, F, C> Command<C> for ParsedCommand<E, F>
where
    E: Tuple,
    F: Func<
        <<<C as Tuple>::HList as Combine<<E as Tuple>::HList>>::Output as HList>::Tuple
    >,
    C: Tuple,
    <C as Tuple>::HList: Combine<<E as Tuple>::HList>
{
    fn call(self, ctx: C) {
        let ex = self.extracted.hlist();
        let ctx = ctx.hlist();
        let args = ctx.combine(ex).flatten();
        self.command.call(args);
    }
}