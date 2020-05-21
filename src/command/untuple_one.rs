use super::{Input, Tuple, Command, CommandBase, Context};

#[derive(Clone, Copy, Debug)]
pub struct UntupleOne<F> {
    pub(super) command: F,
}

impl<F, T> CommandBase for UntupleOne<F>
where
    F: Command<Argument = (T,)>,
    T: Tuple,
{
    type Argument = T;
    type Context = F::Context;

    #[inline]
    fn call<'i>(&self, ctx: &mut Self::Context, input: &mut Input<'i>) -> Result<Self::Argument, <Self::Context as Context>::Error> {
        match self.command.call(ctx, input) {
            Ok((arg,)) => Ok(arg),
            Err(err) => Err(err),
        }
    }
}