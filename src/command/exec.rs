use super::{CommandError, Command, CommandBase, Context, Func, Input};
#[derive(Copy, Clone, Debug)]
pub struct Exec<T, F> {
    pub(super) command: T,
    pub(super) callback: F,
}

impl<T, F> CommandBase for Exec<T, F>
where
    T: Command,
    F: Func<
        T::Argument,
        Output = Result<<T::Context as Context>::Ok, <T::Context as Context>::Error>,
    >,
{
    type Argument = (<Self::Context as Context>::Ok,);
    type Context = T::Context;

    fn call<'i>(
        &self,
        ctx: &mut Self::Context,
        input: &mut Input<'i>,
    ) -> Result<Self::Argument, <T::Context as Context>::Error> {
        match (self.command.call(ctx, input), input.is_empty()) {
            (Ok(ex), true) => self.callback.call(ex).map(|ok| (ok,)),
            (Err(err), _) => Err(err),
            _ => Err(CommandError::NotFound.into()),
        }
    }
}
