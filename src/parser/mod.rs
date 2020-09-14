mod then;
mod map;
mod alt;
mod unify;
mod untuple_one;
mod executor;

pub(crate) use self::then::Then;
pub(crate) use self::map::Map;
pub(crate) use self::alt::Alt;
pub(crate) use self::unify::Unify;
pub(crate) use self::untuple_one::UntupleOne;
pub(crate) use self::executor::Executor;

pub(crate) use crate::generic::{Combine, Either, Func, HList, Tuple};

pub use crate::Input;

use std::borrow::Cow;

#[derive(Debug)]
pub enum Error {
    // Exact
    Literal(Cow<'static, str>),
    // OneOf
    Literals(Vec<Cow<'static, str>>),
    Todo,
    UnkownCommand,
}

pub type Result<Extract> = ::std::result::Result<Extract, Error>;

pub trait ParserBase {
    type Extract: Tuple;

    fn parse<'i>(&self, input: &mut Input<'i>) -> Result<Self::Extract>;
}

pub trait Parser: ParserBase {
    fn then<F>(self, other: F) -> Then<Self, F>
    where
        Self: Sized,
        <Self::Extract as Tuple>::HList: Combine<<F::Extract as Tuple>::HList>,
        F: Parser + Clone,
    {
        Then {
            first: self,
            second: other,
        }
    }

    /// Alternative parser
    fn alt<F>(self, other: F) -> Alt<Self, F>
    where
        Self: Sized,
        F: Parser,
    {
        Alt {
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

    fn boxed<'a>(self) -> Box<dyn Parser<Extract = Self::Extract>>
    where
        Self: Sized,
        Self: 'static,
    {
        Box::new(self)
    }

    fn execute<S, F>(self, fun: F) -> Executor<Self, S, F>
    where
        Self: Sized,
        <Self::Extract as Tuple>::HList: Combine<S::HList>,
        S: Tuple,
        F: Func<<<<Self::Extract as Tuple>::HList as Combine<S::HList>>::Output as HList>::Tuple>
            + Clone,
    {
        Executor {
            parser: self,
            state: Default::default(),
            callback: fun,
        }
    }
}

impl<T> Parser for T where T: ParserBase {}


#[cfg(test)]
mod tests {
    use super::ParserBase;
    use crate::generic::{FuncOnce, Func};
    use crate::{Parser, CommandDispatcher, State, RefMut};
    use crate::parsers::{Literals, any, param, literal};

    #[test]
    fn and_command() {
        let root = literal("hello")
            .then(literal("world"))
            .then(param())
            .map(|a: i32| move |n: &mut i32| *n += a);

        let state = 69;

        let foo = literal("foo")
            .then(param())
            .execute(|_a: u32, _state: i32| {})
            .alt(literal("boo").execute(|_state: i32| {}));

        if let Ok((command,)) = foo.parse(&mut "foo -32".into()) {
            command.call((state,));
        }

        let mut n = 45;

        if let Ok((command,)) = root.parse(&mut "Hello World -3".into()) {
            command(&mut n)
        }

        assert_eq!(n, 42);

        let command = root.parse(&mut "bar".into());
        assert!(command.is_err());
    }

    struct PlayerState(i32, i32, i32);

    impl State for PlayerState {
        fn get<T>(&self) -> Option<T> {
            todo!()
        }
    }

    #[test]
    fn new_api() {
        let mut dispatcher = CommandDispatcher::default();
        dispatcher.register(
            "tp",
            param::<i32>()
                .then(param::<i32>())
                .then(param::<i32>())
                .execute::<((_, _, _),), _>(|x: i32, y: i32, z: i32, (player_x, player_y, player_z): (RefMut<i32>, RefMut<i32>, RefMut<i32>)| {
                    // player_x.as_mut() = x;
                    // player_y.as_mut() = y;
                    // player_z.as_mut() = z;     
                }),
        );

        let result = dispatcher.call(PlayerState(0, 0, 0), "tp 10 20 30");
        dbg!(&result);
        assert!(result.is_ok())
    }

    #[test]
    fn or_command() {
        let add = literal("add")
            .then(param())
            .map(|n: i32| move |state: &mut i32| *state += n);

        let times = literal("times")
            .then(param())
            .map(|n: i32| move |state: &mut i32| *state *= n);

        let reset = literal("reset").map(|| |state: &mut i32| *state = 0);
        let root = literal("math").then(add.alt(times).alt(reset));

        let mut n = 45;

        if let Ok((command,)) = root.parse(&mut "math add 10".into()) {
            command.call((&mut n,))
        }
        assert_eq!(n, 55);

        if let Ok((command,)) = root.parse(&mut "math times 10".into()) {
            command.call((&mut n,))
        }
        assert_eq!(n, 550);

        if let Ok((command,)) = root.parse(&mut "math reset".into()) {
            command.call((&mut n,))
        }
        assert_eq!(n, 0);

        let command = root.parse(&mut "foo".into());
        assert!(command.is_err());
    }

    #[test]
    fn hashed_literals() {
        let mut root: Literals<()> = Literals::default();
        root.insert("a0", any().boxed());
        root.insert("a1", any().boxed());
        root.insert("a2", any().boxed());
        root.insert("a3", any().boxed());
        root.insert("a4", any().boxed());
        root.insert("a5", any().boxed());
        root.insert("a6", any().boxed());
        root.insert("a7", any().boxed());
        root.insert("a8", any().boxed());
        root.insert("a9", any().boxed());

        assert!(root.parse(&mut "a1".into()).is_ok())
    }
}
