use super::{Command, CommandBase, Func, Input};
#[derive(Copy, Clone, Debug)]
pub struct Exec<T, F> {
    pub(super) command: T,
    pub(super) callback: F,
}

impl<T, F> CommandBase for Exec<T, F>
where
    T: Command,
    F: Func<T::Argument, Output = Result<(), ()>> + Clone + Send,
{
    type Argument = ();

    fn call<'i>(&self, input: &mut Input<'i>) -> Result<(), ()> {
        match (self.command.call(input), input.is_empty()) {
            (Ok(ex), true) => self.callback.call(ex),
            _ => Err(())
        }
    }
}