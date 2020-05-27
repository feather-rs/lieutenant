mod and;
mod input;
mod map;
mod or;
mod unify;
mod untuple_one;

pub(crate) use self::and::And;
pub(crate) use self::map::Map;
pub(crate) use self::or::Or;
pub(crate) use self::unify::Unify;
pub(crate) use self::untuple_one::UntupleOne;
use crate::generic::{Combine, Either, Func, HList, Tuple};
pub use input::Input;
use unicase::UniCase;

pub trait ParserBase {
    type Extract: Tuple;

    fn parse<'i>(&self, input: &mut Input<'i>) -> Option<Self::Extract>;
}

pub trait Parser: ParserBase {
    fn then<F>(self, other: F) -> And<Self, F>
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

    fn map<F>(self, fun: F) -> Map<Self, F>
    where
        Self: Sized,
        F: Func<Self::Extract> + Clone,
    {
        Map {
            parser: self,
            callback: fun,
        }
    }

    fn untuple_one<T>(self) -> UntupleOne<Self>
    where
        Self: Parser<Extract = (T,)> + Sized,
        T: Tuple,
    {
        UntupleOne { parser: self }
    }

    fn unify<T>(self) -> Unify<Self>
    where
        Self: Parser<Extract = (Either<T, T>,)> + Sized,
        T: Tuple,
    {
        Unify { parser: self }
    }
}

impl<T> Parser for T where T: ParserBase {}

#[derive(Debug, Clone)]
pub struct Literal<L> {
    value: UniCase<L>,
}

impl<L> AsRef<str> for Literal<L>
where
    L: AsRef<str>
{
    fn as_ref(&self) -> &str {
        self.value.as_ref()
    }
}

impl<L> ParserBase for Literal<L>
where
    L: AsRef<str>
{
    type Extract = ();

    fn parse<'i>(&self, input: &mut Input<'i>) -> Option<Self::Extract> {
        let head = input.advance_until(" ");
        let head = &UniCase::new(head);
        let value = &self.value;
        if value == head {
            Some(())
        } else {
            None
        }
    }
}

pub fn literal<L: AsRef<str>>(lit: L) -> Literal<L> {
    Literal { value: UniCase::new(lit) }
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

#[derive(Clone)]
pub struct Any;

impl ParserBase for Any {
    type Extract = ();

    #[inline]
    fn parse<'i>(&self, _input: &mut Input<'i>) -> Option<Self::Extract> {
        Some(())
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
        let root = literal("hello")
            .then(literal("world"))
            .then(param())
            .map(|a: i32| move |n: &mut i32| *n += a);

        let mut n = 45;

        if let Some((command,)) = root.parse(&mut "Hello World -3".into()) {
            command(&mut n)
        }

        assert_eq!(n, 42);

        let command = root.parse(&mut "bar".into());
        assert!(command.is_none());
    }

    #[test]
    fn or_command() {
        let root = literal("hello")
            .then(literal("world"))
            .map(|| |n: &mut i32| *n = 42)
            .or(literal("foo")
                .then(param())
                .map(|a: i32| move |n: &mut i32| *n += a))
            .or(literal("bar").map(|| |_: &mut i32| {}));

        let mut n = 45;

        if let Some((command,)) = root.parse(&mut "Hello World".into()) {
            command.call((&mut n,))
        }

        assert_eq!(n, 42);

        if let Some((command,)) = root.parse(&mut "foo 10".into()) {
            command.call((&mut n,))
        }

        assert_eq!(n, 52);

        let command = root.parse(&mut "bar".into());
        assert!(command.is_some());
    }

    #[test]
    fn async_command() {
        let root = literal("foo")
            .then(param())
            .map(|a: i32| move |n: i32| async move { n + a });

        if let Some((command,)) = root.parse(&mut "foo 10".into()) {
            let res = smol::run(command(0));
            assert_eq!(res, 10)
        }
    }
}
