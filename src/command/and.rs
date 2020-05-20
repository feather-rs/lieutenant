use super::{Combine, Command, CommandBase, HList, Input, Tuple};

#[derive(Debug, Copy, Clone)]
pub struct And<T, U> {
    pub(super) first: T,
    pub(super) second: U,
}

impl<T, U> CommandBase for And<T, U>
where
    T: Command,
    T::Argument: Send,
    U: Command + Clone + Send,
    <T::Argument as Tuple>::HList: Combine<<U::Argument as Tuple>::HList> + Send,
{
    type Argument = <<<T::Argument as Tuple>::HList as Combine<<U::Argument as Tuple>::HList>>::Output as HList>::Tuple;

    fn call<'i>(&self, input: &mut Input<'i>) -> Result<Self::Argument, ()> {
        Ok(self.first.call(input)?.hlist().combine(self.second.call(input)?.hlist()).flatten())
    }
}