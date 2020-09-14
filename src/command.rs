use crate::generic::{Combine, Either, Func, FuncOnce, HList, Tuple};
use crate::parser::Realize;

pub struct Ref<T>(T);
pub struct RefMut<T>(T);

pub trait State {
    // TODO handle option such to avoide unwrap
    fn get<T>(&self) -> Option<T>;
}

pub trait Command {
    type Output;
    fn call<S>(self, state: S) -> Self::Output
    where
        S: State;
}

impl<A, B> Command for Either<A, B>
where
    A: Command,
    B: Command<Output = A::Output>,
{
    type Output = A::Output;

    fn call<S>(self, state: S) -> Self::Output
    where
        S: State,
    {
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
    A: Tuple,
    F: Func<A>,
{
    type Output = F::Output;

    fn call<T>(self, state: T) -> Self::Output
    where
        T: State,
    {
        let arguments = todo!(); // self.arguments.realize(state);

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

impl<A, F> FuncOnce<A::State> for CommandMapping<A, F>
where
    A: Realize,
    F: Func<A::Output>,
{
    type Output = F::Output;

    fn call(self, state: A::State) -> Self::Output {
        let realized_arguments = self.arguments.realize(&state);
        self.callback.call(realized_arguments)
    }
}
