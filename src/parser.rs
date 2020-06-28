use crate::Context;

/// The input type, acting like a stream of characters.
#[derive(Copy, Clone, Debug)]
pub struct Input<'a> {
    ptr: &'a str,
}

impl<'a> Input<'a> {
    pub fn new(ptr: &'a str) -> Self {
        Self { ptr }
    }

    /// Advances the pointer until the given pattern has been reached, returning
    /// the consumed characters.
    pub fn advance_until<'b>(&'b mut self, pat: &str) -> &'a str {
        let head = self.ptr.split(pat).next().unwrap_or("");
        self.ptr = &self.ptr[(head.len() + pat.len()).min(self.ptr.len())..];
        head
    }

    /// Advances until the end of input, returning all
    /// consumed characters.
    pub fn advance_to_end(&mut self) -> &'a str {
        let consumed = self.ptr;
        self.ptr = &self.ptr[self.ptr.len()..];
        consumed
    }

    /// Returns the number of remaining characters to read.
    pub fn len(&self) -> usize {
        self.ptr.len()
    }

    /// Returns whether there are no more characters to read.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Denotes a type which can be used as a command _argument_.
///
/// The type must define the following functions:
/// * `satisfies`, returning whether the given input is
/// a valid instance of this argument.
pub trait ArgumentKind<C: Context>: Sized {
    /// The error type returned by `Parse`.
    ///
    /// Must implement `Into<C::Error>`.
    type ParseError: Into<C::Error>;

    /// Returns whether the given input is a valid
    /// instance of this argument. Should advance the
    /// pointer to `input` by the number of characters read.
    ///
    /// This can be performed conveniently using the `ParserUtil`
    /// trait.
    fn satisfies<'a>(ctx: &C, input: &mut Input<'a>) -> bool;

    /// Parses a value of this type from the given stream of characters.
    ///
    /// Should advance the pointer to `input` by the number of characters read.
    fn parse<'a>(ctx: &C, input: &mut Input<'a>) -> Result<Self, Self::ParseError>;
}

pub type SatisfiesFn<C> = fn(&C, &mut Input) -> bool;

mod arguments {
    use super::*;
    use std::num::*;
    use std::path::PathBuf;
    use std::str::FromStr;

    macro_rules! from_str_argument {
        ($($ty:ty,)* $(,)?) => {
            $(
                impl <C> ArgumentKind<C> for $ty where C: Context, C::Error: From<<$ty as FromStr>::Err> {
                    type ParseError = <$ty as FromStr>::Err;

                    fn satisfies<'a>(ctx: &C, input: &mut Input<'a>) -> bool {
                        Self::parse(ctx, input).is_ok()
                    }

                    fn parse<'a>(_ctx: &C, input: &mut Input<'a>) -> Result<Self, Self::ParseError> {
                        let head = input.advance_until(" ");
                        Ok(Self::from_str(head)?)
                    }
                }
            )*
        }
    }

    from_str_argument!(
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

#[cfg(test)]
mod tests {
    use crate::Input;

    #[test]
    fn input_advance_until() {
        let command = "cmd";
        let values = ["value1", "value2 with spaces", "value3"];

        let args = [command, &values.join(", ")].join(" ");
        let mut input = Input::new(&args);

        assert_eq!(input.advance_until(" "), command);
        assert_eq!(input.advance_until(", "), values[0]);
        assert_eq!(input.advance_until(", "), values[1]);
        assert_eq!(input.advance_until("INVALID"), values[2]);
        assert_eq!(input.advance_until(" "), "");
    }

    #[test]
    fn input_advance_to_end() {
        let input_str = "first; second and third";
        let mut input = Input::new(input_str);

        assert_eq!(input.advance_to_end(), input_str);
        assert!(input.is_empty());
        assert_eq!(input.advance_to_end(), "");
        assert!(input.is_empty());
    }

    #[test]
    fn input_len() {
        let input_str = "first :: second :: third";
        let mut input = Input::new(input_str);
        let pattern = " :: ";

        assert_eq!(input.len(), input_str.len());

        let first = input.advance_until(pattern);
        assert_eq!(first, "first");
        assert_eq!(input.len(), input_str.len() - (first.len() + pattern.len()));

        let second = input.advance_until(pattern);
        assert_eq!(second, "second");
        assert_eq!(
            input.len(),
            input_str.len() - (first.len() + second.len() + 2 * pattern.len())
        );

        let third = input.advance_to_end();
        assert_eq!(third, "third");
        assert_eq!(
            input.len(),
            input_str.len() - (first.len() + second.len() + third.len() + 2 * pattern.len())
        );
    }

    #[test]
    fn input_empty() {
        let input_str = "first,second,third";
        let mut input = Input::new(input_str);

        assert!(!input.is_empty());
        input.advance_until(",");
        assert!(!input.is_empty());
        input.advance_until(",");
        assert!(!input.is_empty());
        input.advance_to_end();
        assert!(input.is_empty());
    }
}
