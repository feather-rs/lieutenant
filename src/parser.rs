use std::any::Any;

pub trait ArgumentChecker: Any {
    fn satisfies(&self, input: &str) -> bool;
    fn equals(&self, other: &dyn Any) -> bool;
}

pub trait ArgumentParser {
    type Output;

    fn parse(&self, input: &str) -> anyhow::Result<Self::Output>;
    fn default() -> Self
    where
        Self: Sized;
}

pub trait ArgumentKind: Sized {
    type Checker: ArgumentChecker;
    type Parser: ArgumentParser<Output = Self>;
}

macro_rules! from_str_checker {
    ($($ty:ident: $checker:ident, $parser:ident,)*) => {
        $(
            #[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
            #[doc(hidden)]
            pub struct $checker;

            impl ArgumentChecker for $checker {
                fn satisfies(&self, input: &str) -> bool {
                    <$ty as std::str::FromStr>::from_str(input).is_ok()
                }

                fn equals(&self, other: &dyn Any) -> bool {
                    other.downcast_ref::<$checker>().is_some()
                }
            }

            #[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
            #[doc(hidden)]
            pub struct $parser;

            impl ArgumentParser for $parser {
                type Output = $ty;

                fn parse(&self, input: &str) -> anyhow::Result<Self::Output> {
                    <$ty as std::str::FromStr>::from_str(input).map_err(anyhow::Error::from)
                }

                fn default() -> Self {
                    <Self as Default>::default()
                }
            }

            impl ArgumentKind for $ty {
                type Checker = $checker;
                type Parser = $parser;
            }
        )*
    }
}

pub mod parsers {
    use super::*;

    from_str_checker!(
        i32: I32Checker,
        I32Parser,
        u32: U32Checker,
        U32Parser,
        String: StringChecker,
        StringParser,
    );
}
