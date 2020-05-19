mod and;
mod exec;
mod or;

pub(crate) use self::and::And;
pub(crate) use self::exec::Exec;
pub(crate) use self::or::Or;
use crate::generic::{Combine, Func, HList, Tuple};
pub use crate::{Context, Input};
use futures::future;
use std::future::Future;

pub trait CommandBase<C: Context> {
    type Argument: Tuple;
    type Future: Future<Output = Result<Self::Argument, ()>>;

    fn parse(&self, ctx: *mut C, input: *mut Input) -> Self::Future;
}

pub trait Command<C: Context>: CommandBase<C> {
    fn and<F>(self, other: F) -> And<Self, F>
    where
        C: Context,
        Self: Sized,
        <Self::Argument as Tuple>::HList: Combine<<F::Argument as Tuple>::HList>,
        F: Command<C> + Clone,
    {
        And {
            first: self,
            second: other,
        }
    }

    fn or<F>(self, other: F) -> Or<Self, F>
    where
        Self: Sized,
        F: Command<C>,
    {
        Or {
            first: self,
            second: other,
        }
    }

    fn exec<F>(self, func: F) -> Exec<Self, F>
    where
        Self: Sized,
        F: Func<Self::Argument> + Clone,
    {
        Exec {
            command: self,
            callback: func,
        }
    }
}

impl<T, C: Context> Command<C> for T where T: CommandBase<C> {}

#[derive(Debug, Clone)]
pub struct Literal<L>(L);

impl<L, C> CommandBase<C> for Literal<L>
where
    C: Context,
    L: AsRef<str>,
{
    type Argument = ();
    type Future = future::Ready<Result<Self::Argument, ()>>;

    fn parse(&self, _ctx: *mut C, input: *mut Input) -> Self::Future {
        let input = unsafe { &mut *input };
        if self.0.as_ref() == input.advance_until(" ") {
            future::ready(Ok(()))
        } else {
            future::ready(Err(()))
        }
    }
}

pub fn literal<L: AsRef<str>>(lit: L) -> Literal<L> {
    Literal(lit)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct State;

    impl Context for State {}

    #[test]
    fn simple_command() {
        let command = literal("hello")
            .and(literal("world"))
            .exec(|| println!("Hello world!"));
        let mut input = Input::new("hello world");
        smol::run(async {
            let result = command.parse(&mut State, &mut input).await;
            assert_eq!(result, Ok(((),)))
        });

        let mut input = Input::new("hello");
        smol::run(async {
            let result = command.parse(&mut State, &mut input).await;
            assert_eq!(result, Err(()))
        });
    }

    #[test]
    fn multiple_commands() {
        let root = literal("hello")
            .exec(|| println!("hello"))
            .or(literal("world").exec(|| println!("world")));

        let mut input = Input::new("hello");
        smol::run(async {
            let result = root.parse(&mut State, &mut input).await;
        });

        let mut input = Input::new("world");
        smol::run(async {
            let result = root.parse(&mut State, &mut input).await;
        });

        let mut input = Input::new("foo");
        smol::run(async {
            let result = root.parse(&mut State, &mut input).await;
        });
    }

    #[test]
    fn multiple_exec() {
        let command = literal("hello")
            .exec(|| 32i64)
            .exec(|n| (n, n + 2))
            .exec(|(a, b)| println!("{} {}", a, b));
    }
}
