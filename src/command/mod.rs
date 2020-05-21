mod and;
mod exec;
mod or;
mod untuple_one;

pub(crate) use self::and::And;
pub(crate) use self::exec::Exec;
pub(crate) use self::or::Or;
pub(crate) use self::untuple_one::UntupleOne;
use crate::generic::{Combine, Func, HList, Tuple};
pub use crate::{Context, Input};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommandError {
    #[error("could not find the given command")]
    NotFound,
}

pub trait CommandBase {
    type Argument: Tuple;
    type Context: Context;

    fn call<'i>(
        &self,
        ctx: &mut Self::Context,
        input: &mut Input<'i>,
    ) -> Result<Self::Argument, <Self::Context as Context>::Error>;
}

pub trait Command: CommandBase {
    fn and<F>(self, other: F) -> And<Self, F>
    where
        Self: Sized,
        <Self::Argument as Tuple>::HList: Combine<<F::Argument as Tuple>::HList>,
        F: Command + Clone,
    {
        And {
            first: self,
            second: other,
        }
    }

    fn or<F>(self, other: F) -> Or<Self, F>
    where
        Self: Sized,
        F: Command,
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

    fn untuple_one<T>(self) -> UntupleOne<Self>
    where
        Self: Command<Argument = (T,)> + Sized,
        T: Tuple,
    {
        UntupleOne { command: self }
    }
}

impl<T> Command for T where T: CommandBase {}

#[derive(Debug, Clone)]
pub struct Literal<C> {
    value: &'static str,
    context: std::marker::PhantomData<C>,
}

impl<C: Context> AsRef<str> for Literal<C> {
    fn as_ref(&self) -> &str {
        self.value
    }
}

impl<C> CommandBase for Literal<C>
where
    C: Context,
{
    type Argument = ();
    type Context = C;

    fn call<'i>(&self, _ctx: &mut C, input: &mut Input<'i>) -> Result<Self::Argument, <Self::Context as Context>::Error> {
        let head = input.advance_until(" ").to_lowercase();
        let value = self.as_ref().to_lowercase();
        if value == head {
            Ok(())
        } else {
            Err(CommandError::NotFound.into())
        }
    }
}

pub fn literal<C: Context>(lit: &'static str) -> Literal<C> {
    Literal {
        value: lit,
        context: Default::default(),
    }
}

#[derive(Debug, Clone)]
pub struct Any<C>(std::marker::PhantomData<C>);

impl<C: Context> CommandBase for Any<C> {
    type Argument = ();
    type Context = C;

    fn call<'i>(&self, _ctx: &mut C, _input: &mut Input<'i>) -> Result<Self::Argument, <Self::Context as Context>::Error> {
        Ok(())
    }
}

pub fn any<C: Context>() -> Any<C> {
    Any(Default::default())
}

#[derive(Clone)]
pub struct Provider<C, T> {
    provider: fn(&mut C) -> T,
}

impl<C, T> CommandBase for Provider<C, T>
where
    C: Context,
{
    type Argument = (T,);
    type Context = C;

    fn call<'i>(&self, ctx: &mut C, _input: &mut Input<'i>) -> Result<Self::Argument, <Self::Context as Context>::Error> {
        let provider = self.provider;
        Ok((provider(ctx),))
    }
}

pub fn provider<C, T>(provider: fn(&mut C) -> T) -> Provider<C, T> {
    Provider {
        provider,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use thiserror::Error;

    #[derive(Clone)]
    struct State;
    impl Context for State {
        type Error = CommandError;
        type Ok = ();
    }

    #[test]
    fn and_command() {
        let command = literal("hello").and(literal("world")).exec(|| {
            println!("hello world");
            Ok(())
        });

        let res = command.call(&mut State, &mut "hello world".into());

        assert_eq!(res, Ok(()));

        let res = command.call(&mut State, &mut "foo".into());
        assert_eq!(res, Err(CommandError::NotFound))
    }

    #[test]
    fn or_command() {
        let command = literal("hello")
            .exec(|| {
                println!("hello");
                Ok(())
            })
            .or(literal("world").exec(|| {
                println!("world");
                Ok(())
            }));

        let res = command.call(&mut State, &mut "hello".into());
        assert!(res.is_ok());

        let res = command.call(&mut State, &mut "world".into());
        assert!(res.is_ok());

        let res = command.call(&mut State, &mut "foo".into());
        assert!(res.is_err());
    }

    #[test]
    fn provider_command() {
        let command = provider(|ctx: &mut State| ctx.clone()).and(literal("hello")).exec(|_| Ok(()));

        let res = command.call(&mut State, &mut "hello".into());
        assert!(res.is_ok());
    }
}
