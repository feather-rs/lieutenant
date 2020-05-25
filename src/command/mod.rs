mod and;
mod exec;
mod or;
mod untuple_one;

pub(crate) use self::and::And;
pub(crate) use self::exec::{Command, Exec};
pub(crate) use self::or::Or;
pub(crate) use self::untuple_one::UntupleOne;
use crate::generic::{Combine, Func, Either, HList, Tuple};
pub use crate::{Context, Input};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommandError {
    #[error("could not find the given command")]
    NotFound,
}

pub trait ParserBase {
    type Extract: Tuple;

    fn parse<'i>(&self, input: &mut Input<'i>) -> Option<Self::Extract>;
}

pub trait Parser: ParserBase {
    fn and<F>(self, other: F) -> And<Self, F>
    where
        Self: Sized,
        <Self::Extract as Tuple>::HList: Combine<<F::Extract as Tuple>::HList>,
        F: Parser + Clone,
    {
        And {
            first: self,
            second: other,
        }
    }

    fn or<F>(self, other: F) -> Or<Self, F>
    where
        Self: Sized,
        F: Parser,
    {
        Or {
            first: self,
            second: other,
        }
    }

    fn exec<F, C>(self, command: F) -> Exec<Self, F>
    where
        Self: Sized,
        C: Tuple,
        F: Func<
            <<<C as Tuple>::HList as Combine<<Self::Extract as Tuple>::HList>>::Output as HList>::Tuple
        >,
        <C as Tuple>::HList: Combine<<Self::Extract as Tuple>::HList>
    {
        Exec {
            parser: self,
            command: command,
        }
    }

    fn untuple_one<T>(self) -> UntupleOne<Self>
    where
        Self: Parser<Extract = (T,)> + Sized,
        T: Tuple,
    {
        UntupleOne { parser: self }
    }
}

impl<T> Parser for T where T: ParserBase {}

#[derive(Debug, Clone)]
pub struct Literal {
    value: &'static str,
}

impl AsRef<str> for Literal {
    fn as_ref(&self) -> &str {
        self.value
    }
}

impl ParserBase for Literal {
    type Extract = ();

    fn parse<'i>(&self, input: &mut Input<'i>) -> Option<Self::Extract> {
        let head = input.advance_until(" ").to_lowercase();
        let value = self.as_ref().to_lowercase();
        if value == head {
            Some(())
        } else {
            None
        }
    }
}

pub fn literal(lit: &'static str) -> Literal {
    Literal { value: lit }
}

#[derive(Debug, Clone)]
pub struct Param<T> {
    param: std::marker::PhantomData<T>,
}

impl<T> ParserBase for Param<T>
where
    T: std::str::FromStr,
{
    type Extract = (T,);

    fn parse<'i>(&self, input: &mut Input<'i>) -> Option<Self::Extract> {
        let head = input.advance_until(" ");
        match T::from_str(head) {
            Ok(ok) => Some((ok,)),
            Err(_) => None,
        }
    }
}

pub fn param<T: std::str::FromStr>() -> Param<T> {
    Param {
        param: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn and_command() {
        let root = literal("hello")
            .and(literal("world"))
            .exec::<_, (_,)>(|n: &mut i32| *n = 42);

        let mut n = 45;

        if let Some((command,)) = root.parse(&mut "hello world".into()) {
            command.call((&mut n,));
        }

        assert_eq!(n, 42);

        let command = root.parse(&mut "bar".into());
        assert!(command.is_none());
    }

    #[test]
    fn or_command() {
        let root = literal("hello")
            .and(literal("world"))
            .exec::<_, (_,)>(|n: &mut i32| *n = 42)
            .or(literal("foo").and(param()).exec::<_, (_,)>(|n: &mut i32, a: i32| *n += a));

        let mut n = 45;

        if let Some((command,)) = root.parse(&mut "hello world".into()) {
            command.call((&mut n,));
        }

        assert_eq!(n, 42);

        if let Some((command,)) = root.parse(&mut "foo 10".into()) {
            command.call((&mut n,));
        }

        assert_eq!(n, 52);

        let command = root.parse(&mut "bar".into());
        assert!(command.is_none());
    }

    #[test]
    fn guard_command() {
        // let command = guard(|_: &mut State| Ok(())).and(literal("hello")).exec(|| Ok(()));

        // let res = command.parse(&mut State, &mut "hello".into());
        // assert!(res.is_ok());
    }
}
