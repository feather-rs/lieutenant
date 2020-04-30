use std::any::Any;

#[derive(Clone, Debug)]
pub struct Input<'a> {
    cursor: usize,
    value: &'a str,
}

impl<'a> Input<'a> {
    pub fn head(&mut self, pattern: &str) -> &'a str {
        if !self.is_empty() {
            let (_, tail) = self.value.split_at(self.cursor);
            let head = tail.split(pattern).next().unwrap_or("");
            self.cursor += head.len() + pattern.len();
            head
        } else {
            ""
        }
    }

    pub fn tail(&self) -> &str {
        if !self.is_empty() {
            self.value.split_at(self.cursor).1
        } else {
            ""
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cursor >= self.value.len()
    }
}

impl<'a> From<&'a str> for Input<'a> {
    fn from(input: &'a str) -> Self {
        Self {
            cursor: 0,
            value: input,
        }
    }
}

pub trait ArgumentChecker<C>: Any {
    fn satisfies(&self, ctx: &C, input: &mut Input) -> bool;
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

pub trait ArgumentParser<C> {
    type Output;

    fn parse(&self, ctx: &mut C, input: &mut Input) -> anyhow::Result<Self::Output>;
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
        T: FromStr + Clone + 'static,
    {
        fn satisfies(&self, _ctx: &C, input: &mut Input) -> bool {
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
        T: FromStr + 'static,
        <T as FromStr>::Err: std::error::Error + Send + Sync,
    {
        type Output = T;

        fn parse(&self, _ctx: &mut C, input: &mut Input) -> anyhow::Result<Self::Output> {
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

#[cfg(test)]
mod tests {
    use super::Input;
    #[test]
    fn input() {
        let input = Input::from("foo bar");
        {
            let mut input = input.clone();
            let foo = input.head(" ");

            assert_eq!(foo, "foo");
            assert_eq!(input.tail(), "bar");
            assert!(!input.is_empty());

            let bar = input.head(" ");
            assert_eq!(bar, "bar");
            assert!(input.is_empty());
            assert_eq!(input.head(" "), "")
        }

        assert_eq!(input.tail(), "foo bar");
    }
}
