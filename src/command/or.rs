use super::{Context, Command, CommandBase, Input};

#[derive(Clone, Copy, Debug)]
pub struct Or<T, U> {
    pub(super) first: T,
    pub(super) second: U,
}

impl<T, U> CommandBase for Or<T, U>
where
    T: Command,
    U: Command<Argument = T::Argument> + Send + Clone,
{
    type Argument = T::Argument;

    fn call<'i>(&self, input: &mut Input<'i>) -> Result<Self::Argument, ()> {
        match self.first.call(&mut input.clone()) {
            ok @ Ok(_) => ok,
            _ => self.second.call(input),
        }
    }
}
