use std::any::Any;

pub trait Head {
    /// Splits the string on the patterns returning the head and advancing the input to tail.
    fn head<'a, 'b>(&'a mut self, pat: &'b str) -> &'a str;
}

impl Head for &str {
    #[inline]
    fn head<'a, 'b>(&'a mut self, pat: &'b str) -> &'a str {
        let head = self.split(pat).next().unwrap_or("");
        *self = &self[(head.len() + pat.len()).min(self.len())..];
        head
    }
}

pub trait ArgumentChecker<C>: Any + Send + Sync {
    fn satisfies(&self, ctx: &C, input: &mut &str) -> bool;
    /// Returns whether this `ArgumentChecker` will perform
    /// the same operation as some other `ArgumentChecker`.
    ///
    /// This is a workaround for the fact that `PartialEq` and `Eq`
    /// cannot be boxed into trait objects.
    fn equals(&self, other: &dyn Any) -> bool;

    fn default() -> Self
    where
        Self: Sized;

    fn box_clone(&self) -> Box<dyn ArgumentChecker<C>>;
}

pub trait ArgumentParser<C>: Send + Sync {
    type Output;

    fn parse(&self, ctx: &mut C, input: &mut &str) -> anyhow::Result<Self::Output>;
    fn default() -> Self
    where
        Self: Sized;
}

pub trait ArgumentKind<C>: Sized + Send {
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
        T: FromStr + Clone + Send + Sync + 'static,
    {
        fn satisfies(&self, _ctx: &C, input: &mut &str) -> bool {
            let head = input.head(" ");
            T::from_str(head).is_ok()
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

        fn box_clone(&self) -> Box<dyn ArgumentChecker<C>> {
            Box::new(self.clone())
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
        T: FromStr + Send + Sync + 'static,
        <T as FromStr>::Err: std::error::Error + Send + Sync,
    {
        type Output = T;

        fn parse(&self, _ctx: &mut C, input: &mut &str) -> anyhow::Result<Self::Output> {
            let head = input.head(" ");
            Ok(T::from_str(head)?)
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
