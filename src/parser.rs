use crate::Context;
use async_trait::async_trait;
use std::any::Any;
use std::borrow::Cow;

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

#[async_trait]
pub trait ArgumentChecker<C>: Any + Send + Sync + 'static {
    async fn satisfies<'a, 'b>(&self, ctx: &C, input: &'a mut &'b str) -> bool;
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

#[async_trait]
pub trait ArgumentParser<C: Context>: Send + Sync + 'static {
    type Output: Send;

    async fn parse<'a, 'b>(
        &self,
        ctx: &mut C,
        input: &'a mut &'b str,
    ) -> Result<Self::Output, C::Error>;

    fn default() -> Self
    where
        Self: Sized;
}

#[async_trait]
pub trait ArgumentSuggester<C>: Send + Sync
where
    C: Context,
{
    async fn suggestions<'a, 'b, 'c>(
        &'a self,
        _ctx: &'b C,
        _input: &'c str,
    ) -> Vec<Cow<'static, str>>;
}

#[allow(clippy::trivially_copy_pass_by_ref)] // bug in async-trait
#[async_trait]
impl<C: Context> ArgumentSuggester<C> for () {
    async fn suggestions<'a, 'b, 'c>(
        &'a self,
        _ctx: &'b C,
        _input: &'c str,
    ) -> Vec<Cow<'static, str>> {
        Vec::new()
    }
}

pub trait ArgumentKind<C: Context>: Sized + Send {
    type Checker: ArgumentChecker<C>;
    type Suggester: ArgumentSuggester<C>;
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

    #[async_trait]
    impl<C, T> ArgumentChecker<C> for FromStrChecker<T>
    where
        T: FromStr + Clone + Send + Sync + 'static,
        C: Context,
    {
        async fn satisfies<'a, 'b>(&self, _ctx: &C, input: &'a mut &'b str) -> bool {
            let head = input.advance_until(" ");
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

    #[async_trait]
    impl<C, T> ArgumentParser<C> for FromStrParser<T>
    where
        C: Context,
        C::Error: From<<T as FromStr>::Err>,
        T: FromStr + Send + Sync + 'static,
    {
        type Output = T;

        async fn parse<'a, 'b>(
            &self,
            _ctx: &mut C,
            input: &'a mut &'b str,
        ) -> Result<Self::Output, C::Error> {
            let head = input.advance_until(" ");
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
        ($($ty:ty,)*) => {
            $(
                impl<C> ArgumentKind<C> for $ty
                where
                    C: Context,
                    C::Error: From<<$ty as FromStr>::Err>,
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
