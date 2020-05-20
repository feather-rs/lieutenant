use super::{Command, CommandBase, Func, Input};
#[derive(Copy, Clone, Debug)]
pub struct Exec<T, F> {
    pub(super) command: T,
    pub(super) callback: F,
}

impl<T, F> CommandBase for Exec<T, F>
where
    T: Command,
    F: Func<T::Argument, Output = Result<(), ()>>,
{
    type Argument = ();
    type Context = T::Context;

    fn call<'i>(&self, ctx: &mut Self::Context, input: &mut Input<'i>) -> Result<(), ()> {
        match (self.command.call(ctx, input), input.is_empty()) {
            (Ok(ex), true) => self.callback.call(ex),
            _ => Err(()),
        }
    }
}
