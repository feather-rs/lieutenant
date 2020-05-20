mod and;
mod exec;
mod or;

pub(crate) use self::and::And;
pub(crate) use self::exec::Exec;
pub(crate) use self::or::Or;
use crate::generic::{Combine, Func, HList, Tuple};
pub use crate::{Context, Input};

pub trait CommandBase {
    type Argument: Tuple;

    fn call<'i>(&self, input: &mut Input<'i>) -> Result<Self::Argument, ()>;
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
}

impl<T> Command for T where T: CommandBase {}

#[derive(Debug, Clone)]
pub struct Literal {
    value: &'static str,
}

impl AsRef<str> for Literal {
    fn as_ref(&self) -> &str {
        self.value
    }
}

impl CommandBase for Literal {
    type Argument = ();

    fn call<'i>(&self, input: &mut Input<'i>) -> Result<Self::Argument, ()> {
        let head = input.advance_until(" ").to_lowercase();
        let value = self.as_ref().to_lowercase();
        if value == head {
            Ok(())
        } else {
            Err(())
        }
    }
}

pub fn literal(lit: &'static str) -> Literal {
    Literal { value: lit }
}

#[derive(Debug, Clone)]
pub struct Any;

impl CommandBase for Any {
    type Argument = ();

    fn call<'i>(&self, _input: &mut Input<'i>) -> Result<Self::Argument, ()> {
        Ok(())
    }
}

pub fn any() -> Any {
    Any
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn and_command() {
        let command = literal("hello").and(literal("world")).exec(|| {
            println!("hello world");
            Ok(())
        });

        let res = command.call(&mut "hello world".into());
        assert_eq!(res, Ok(()));

        let res = command.call(&mut "foo".into());
        assert_eq!(res, Err(()))
    }

    #[test]
    fn or_command() {
        let command = literal("hello").exec(|| {
            println!("hello");
            Ok(())
        }).or(literal("world").exec(|| {
            println!("world");
            Ok(())
        }));

        let res = command.call(&mut "hello".into());
        assert_eq!(res, Ok(()));

        let res = command.call(&mut "world".into());
        assert_eq!(res, Ok(()));

        let res = command.call(&mut "foo".into());
        assert_eq!(res, Err(()))
    }
}
