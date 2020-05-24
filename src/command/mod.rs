mod and;
mod or;
mod untuple_one;
mod exec;

pub(crate) use self::and::And;
pub(crate) use self::or::Or;
pub(crate) use self::untuple_one::UntupleOne;
pub(crate) use self::exec::Exec;
use crate::generic::{Combine, Func, HList, Tuple};
pub use crate::{Context, Input};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommandError {
    #[error("could not find the given command")]
    NotFound,
}

pub trait ParserBase {
    type Extract: Tuple;

    fn parse<'i>(
        &self,
        input: &mut Input<'i>,
    ) -> Option<Self::Extract>;
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

    fn exec<'a, C>(self, command: fn(&'a mut C, Self::Extract) -> ()) -> Exec<'a, Self, C>
    where
        Self: Sized,
        Self::Extract: 'static,
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
    Literal {
        value: lit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn and_command() {
        let mut n = 45;

        {
            let command = literal("hello").and(literal("world")).exec(|n: &mut i32, ()| {
                *n = 42;
            });

            let res = command.parse( &mut "hello world".into());
            if let Some((command,)) = res {
                command.call(&mut n);
            }
        }

        assert_eq!(n, 42);

        // let res = command.parse(&mut "foo".into());
        // assert_eq!(res, None)
    }

    #[test]
    fn or_command() {
        // let command = literal("hello")
        //     .exec(|| {
        //         println!("hello");
        //         Ok(())
        //     });

        // let res = command.call(&mut State, &mut "hello".into());
        // assert_eq!(res, Ok(()));

        // let res = command.call(&mut State, &mut "world".into());
        // assert_eq!(res, Ok(()));

        // let res = command.call(&mut State, &mut "foo".into());
        // assert_eq!(res, Err(CommandError::NotFound));
    }

    #[test]
    fn guard_command() {
        // let command = guard(|_: &mut State| Ok(())).and(literal("hello")).exec(|| Ok(()));

        // let res = command.parse(&mut State, &mut "hello".into());
        // assert!(res.is_ok());
    }
}
