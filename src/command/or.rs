use super::{Command, CommandBase, Input};

#[derive(Clone, Copy, Debug)]
pub struct Or<T, U> {
    pub(super) first: T,
    pub(super) second: U,
}

impl<T, U> CommandBase for Or<T, U>
where
    T: Command,
    U: Command<Context = T::Context, Argument = T::Argument>,
{
    type Argument = T::Argument;
    type Context = T::Context;

    fn call<'i>(
        &self,
        ctx: &mut Self::Context,
        input: &mut Input<'i>,
    ) -> Result<Self::Argument, ()> {
        match self.first.call(ctx, &mut input.clone()) {
            ok @ Ok(_) => ok,
            _ => self.second.call(ctx, input),
        }
    }
}
