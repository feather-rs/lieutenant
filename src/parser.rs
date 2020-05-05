use crate::{CommandResult, Context};

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
    /// Returns whether the given input may be valid
    /// instance of this argument. Should advance the
    /// pointer to `input` by the number of characters read.
    fn may_satisfy(input: &mut Input) -> bool;

    /// Parses a value of this type from the given stream of characters.
    ///
    /// Should advance the pointer to `input` by the number of characters read.
    fn parse<'a>(ctx: &C, input: &mut Input<'a>) -> CommandResult<C, Self>;

    /// Returns the payload of this parser, specific to the context type.
    ///
    /// The return value can be used to serialize a command graph, for example.
    fn payload() -> C::ArgumentPayload;
}

pub type MaySatisfyFn = fn(&mut Input) -> bool;

mod arguments {
    use super::*;
    use std::str::FromStr;

    macro_rules! from_str_argument {
        ($($ty:ty: $id:ident),* $(,)?) => {
            $(
                impl <C> ArgumentKind<C> for $ty where C: Context, C::Error: From<<$ty as FromStr>::Err> {
                    fn may_satisfy<'a>(input: &mut Input<'a>) -> bool {
                        let head = input.advance_until(" ");
                        Self::from_str(head).is_ok()
                    }

                    fn parse<'a>(_ctx: &C, input: &mut Input<'a>) -> CommandResult<C, Self> {
                        let head = input.advance_until(" ");
                        Ok(Self::from_str(head).map_err(|e| crate::CommandError::Error(e.into()))?)
                    }

                    fn payload() -> C::ArgumentPayload {
                        <C::ArgumentPayload as crate::FromBrigadierId>::from_brigadier_id(crate::BrigadierId::$id)
                    }
                }
            )*
        }
    }

    from_str_argument!(
        i8: Integer,
        i16: Integer,
        i32: Integer,
        i64: Integer,
        i128: Integer,
        isize: Integer,
        u8: Integer,
        u16: Integer,
        u32: Integer,
        u64: Integer,
        u128: Integer,
        usize: Integer,
        f32: Float32,
        f64: Float64,
        String: String,
        bool: Bool,
    );
}
