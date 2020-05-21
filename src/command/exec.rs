use super::{CommandError, Tuple, Combine, HList, Command, CommandBase, Context, Func, Input};
#[derive(Copy, Clone, Debug)]
pub struct Exec<'a, T, F> {
    pub(super) command: T,
    pub(super) callback: F,
    pub(super) marker: std::marker::PhantomData<&'a ()>
}

impl<'a, T, F> CommandBase for Exec<'a, T, F>
where
    T: Command,
    F: Func<
        <<<(&'a mut Self::Context,) as Tuple>::HList as Combine<<Self::Argument as Tuple>::HList>>::Output as HList>::Tuple,
        Output = Result<<Self::Context as Context>::Ok, <Self::Context as Context>::Error>
    > + Clone,
    <(&'a mut Self::Context,) as Tuple>::HList: Combine<<Self::Argument as Tuple>::HList>,
    (&'a mut Self::Context,): Tuple,
    Self::Context: 'a,
{
    type Argument = <Self::Context as Context>::Ok;
    type Context = T::Context;

    fn call<'i>(
        &self,
        ctx: &mut Self::Context,
        input: &mut Input<'i>,
    ) -> Result<Self::Argument, <T::Context as Context>::Error> {
        match (self.command.call(ctx, input), input.is_empty()) {
            (Ok(ex), true) => self.callback.call(ex),
            (Err(err), _) => Err(err),
            _ => Err(CommandError::NotFound.into()),
        }
    }
}
