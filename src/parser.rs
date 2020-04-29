use std::any::Any;

pub trait ArgumentChecker<C>: Any {
    fn satisfies(&self, ctx: &C, input: &str) -> bool;
    /// Returns whether this `ArgumentChecker` will perform
    /// the same operation as some other `ArgumentChecker`.
    ///
    /// This is a workaround for the fact that `PartialEq` and `Eq`
    /// cannot be boxed into trait objects.
    fn equals(&self, other: &dyn Any) -> bool;

    fn default() -> Self
    where
        Self: Sized;
}

pub trait ArgumentParser<C> {
    type Output;

    fn parse(&self, ctx: &mut C, input: &str) -> anyhow::Result<Self::Output>;
    fn default() -> Self
    where
        Self: Sized;
}

pub trait ArgumentKind<C>: Sized {
    type Checker: ArgumentChecker<C>;
    type Parser: ArgumentParser<C, Output = Self>;
}

pub mod parsers {
    use super::*;
    use std::marker::PhantomData;
    use std::str::FromStr;

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct FromStrChecker<T> {
        _phantom: PhantomData<T>,
    }

    impl<T> Default for FromStrChecker<T> {
        fn default() -> Self {
            Self {
                _phantom: PhantomData,
            }
        }
    }

    impl<C, T> ArgumentChecker<C> for FromStrChecker<T>
    where
        T: FromStr + 'static,
    {
        fn satisfies(&self, _ctx: &C, input: &str) -> bool {
            T::from_str(input).is_ok()
        }

        fn equals(&self, other: &dyn Any) -> bool {
            other.downcast_ref::<Self>().is_some()
        }

        fn default() -> Self
        where
            Self: Sized,
        {
            <Self as Default>::default()
        }
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct FromStrParser<T> {
        _phantom: PhantomData<T>,
    }

    impl<T> Default for FromStrParser<T> {
        fn default() -> Self {
            Self {
                _phantom: PhantomData,
            }
        }
    }

    impl<C, T> ArgumentParser<C> for FromStrParser<T>
    where
        T: FromStr + 'static,
        <T as FromStr>::Err: std::error::Error + Send + Sync,
    {
        type Output = T;

        fn parse(&self, _ctx: &mut C, input: &str) -> anyhow::Result<Self::Output> {
            T::from_str(input).map_err(anyhow::Error::from)
        }

        fn default() -> Self
        where
            Self: Sized,
        {
            <Self as Default>::default()
        }
    }

    macro_rules! from_str_argument_kind {
        ($($ty:ty),*) => {
            $(
                impl <C> ArgumentKind<C> for $ty {
                    type Checker = FromStrChecker<Self>;
                    type Parser = FromStrParser<Self>;
                }
            )*
        }
    }

    from_str_argument_kind!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, String, bool);
}
