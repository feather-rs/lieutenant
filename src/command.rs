use crate::generic::{Combine, Either, Func, FuncOnce, HList, Tuple};

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

impl<A, S, F> Command for CommandMapping<A, S, F>
where
    A: Tuple,
    <A as Tuple>::HList: Combine<S::HList>,
    S: Tuple,
    F: Func<<<<A as Tuple>::HList as Combine<S::HList>>::Output as HList>::Tuple>,
{
    type Output = F::Output;

    fn call<T>(self, state: T) -> Self::Output
    where
        T: State,
    {
        // TODO handle option such to avoide unwrap
        self.callback.call(
            self.arguments
                .hlist()
                .combine(state.get::<S>().unwrap().hlist())
                .flatten(),
        )
    }
}

pub struct CommandMapping<A, S, F> {
    arguments: A,
    state: std::marker::PhantomData<S>,
    callback: F,
}

impl<A, S, F> CommandMapping<A, S, F> {
    pub fn new(arguments: A, callback: F) -> Self {
        Self {
            arguments,
            state: Default::default(),
            callback,
        }
    }
}

impl<A, S, F> FuncOnce<S> for CommandMapping<A, S, F>
where
    A: Tuple,
    <A as Tuple>::HList: Combine<S::HList>,
    S: Tuple,
    F: Func<<<<A as Tuple>::HList as Combine<S::HList>>::Output as HList>::Tuple>,
{
    type Output = F::Output;

    fn call(self, args: S) -> Self::Output {
        let combined_args = self.arguments.hlist().combine(args.hlist()).flatten();
        self.callback.call(combined_args)
    }
}
