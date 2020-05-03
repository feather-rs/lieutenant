use crate::Context;
use std::any::Any;
use std::future::Future;
use std::pin::Pin;

pub type FutureBox<'a, O> = Pin<Box<dyn Future<Output = O> + Send + 'a>>;

pub trait ParserUtil {
    /// Advances the pointer until the given pattern and returns head and leaving the tail.
    fn advance_until<'a, 'b>(&'a mut self, pat: &'b str) -> &'a str;
}

impl ParserUtil for &str {
    #[inline]
    fn advance_until<'a, 'b>(&'a mut self, pat: &'b str) -> &'a str {
        let head = self.split(pat).next().unwrap_or("");
        *self = &self[(head.len() + pat.len()).min(self.len())..];
        head
    }
}

pub trait ArgumentChecker<C>: Any + Send + 'static {
    fn satisfies<'a, 'b>(
        &self,
        ctx: &C,
        input: &'a mut &'b str,
    ) -> FutureBox<'a, bool>;
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

pub trait ArgumentParser<C: Context>: Send + 'static {
    type Output: Send;

    fn parse<'a, 'b>(
        &self,
        ctx: &mut C,
        input: &'a mut &'b str,
    ) -> FutureBox<'a, Result<Self::Output, C::Err>>;
    fn default() -> Self
    where
        Self: Sized;
}

pub trait ArgumentSugester<C>
where
    C: Context,
{
    fn suggestions<'a, 'b, 'c>(&'a self, _ctx: &'b C, _input: &'c str) -> FutureBox<'c, Vec<String>>;
}

impl<C: Context> ArgumentSugester<C> for () {
    fn suggestions<'a, 'b, 'c>(&'a self, _ctx: &'b C, _input: &'c str) -> FutureBox<'c, Vec<String>> {
        Box::pin(async {
            Vec::new()
        })
    }
}

pub trait ArgumentKind<C: Context>: Sized + Send {
    type Checker: ArgumentChecker<C>;
    type Suggester: ArgumentSugester<C>;
    type Parser: ArgumentParser<C, Output = Self>;
}

pub mod parsers {
    use super::*;
    use std::marker::PhantomData;
    use std::num::*;
    use std::path::PathBuf;
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
        T: FromStr + Clone + Send + 'static,
    {
        fn satisfies<'a, 'b>(
            &self,
            _ctx: &C,
            input: &'a mut &'b str,
        ) -> FutureBox<'a, bool> {
            Box::pin(async move {
                let head = input.advance_until(" ");
                T::from_str(head).is_ok()
            })
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
        C: Context,
        C::Err: From<<T as FromStr>::Err>,
        T: FromStr + Send + 'static,
    {
        type Output = T;

        fn parse<'a, 'b>(
            &self,
            _ctx: &mut C,
            input: &'a mut &'b str,
        ) -> FutureBox<'a, Result<Self::Output, C::Err>>
        {
            Box::pin(async move {
                let head = input.advance_until(" ");
                Ok(T::from_str(head)?)
            })
        }

        fn default() -> Self
        where
            Self: Sized,
        {
            <Self as Default>::default()
        }
    }

    macro_rules! from_str_argument_kind {
        ($($ty:ty,)*) => {
            $(
                impl<C> ArgumentKind<C> for $ty
                where
                    C: Context,
                    C::Err: From<<$ty as FromStr>::Err>,
                {
                    type Checker = FromStrChecker<Self>;
                    type Suggester = ();
                    type Parser = FromStrParser<Self>;
                }
            )*
        }
    }

    from_str_argument_kind!(
        i8,
        i16,
        i32,
        i64,
        i128,
        isize,
        u8,
        u16,
        u32,
        u64,
        usize,
        f32,
        f64,
        u128,
        String,
        bool,
        char,
        NonZeroI8,
        NonZeroI16,
        NonZeroI32,
        NonZeroI64,
        NonZeroIsize,
        NonZeroU8,
        NonZeroU16,
        NonZeroU32,
        NonZeroU64,
        NonZeroUsize,
        PathBuf,
    );
}
