use super::{Context, Combine, Command, CommandBase, HList, Input, Tuple};

#[derive(Debug, Copy, Clone)]
pub struct And<T, U> {
    pub(super) first: T,
    pub(super) second: U,
}

impl<T, U> CommandBase for And<T, U>
where
    T: Command,
    U: Command<Context = T::Context>,
    <T::Argument as Tuple>::HList: Combine<<U::Argument as Tuple>::HList> + Send,
{
    type Argument = <<<T::Argument as Tuple>::HList as Combine<<U::Argument as Tuple>::HList>>::Output as HList>::Tuple;
    type Context = T::Context;

    fn call<'i>(
        &self,
        ctx: &mut Self::Context,
        input: &mut Input<'i>,
    ) -> Result<Self::Argument, <Self::Context as Context>::Error> {
        Ok(self
            .first
            .call(ctx, input)?
            .hlist()
            .combine(self.second.call(ctx, input)?.hlist())
            .flatten())
    }
}
