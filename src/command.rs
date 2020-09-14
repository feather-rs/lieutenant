use crate::generic::{Combine, Either, Func, FuncOnce, HList, Tuple};
use crate::parser::Lazy;

pub struct Ref<T>(T);
pub struct RefMut<T>(T);

pub trait State {
    // TODO handle option such to avoide unwrap
    fn get<T>(&self) -> Option<T>;
}

pub trait Command {
    type Output;
    type State;

    fn call(self, state: &Self::State) -> Self::Output;
}

impl<A, B> Command for Either<A, B>
where
    A: Command,
    B: Command<Output = A::Output, State = A::State>,
{
    type Output = A::Output;
    type State = A::State;

    fn call(self, state: &Self::State) -> Self::Output {
        match self {
            Either::A(a) => a.call(state),
            Either::B(b) => b.call(state),
        }
    }
}

pub struct CommandMapping<A, F> {
    arguments: A,
    callback: F,
}

impl<A, F> Command for CommandMapping<A, F>
where
    A: Lazy,
    A::Output: Tuple,
    F: Func<A::Output>,
{
    type Output = F::Output;
    type State = A::State;

    fn call(self, state: &Self::State) -> Self::Output {
        let arguments = self.arguments.get(state);

        // TODO handle option such to avoide unwrap
        self.callback.call(arguments)
    }
}

impl<A, F> CommandMapping<A, F> {
    pub fn new(arguments: A, callback: F) -> Self {
        Self {
            arguments,
            callback,
        }
    }
}

impl<A, F> FuncOnce<&mut A::State> for CommandMapping<A, F>
where
    A: Lazy,
    F: Func<A::Output>,
{
    type Output = F::Output;

    fn call(self, state: &mut A::State) -> Self::Output {
        let realized_arguments = self.arguments.get(&state);
        self.callback.call(realized_arguments)
    }
}
