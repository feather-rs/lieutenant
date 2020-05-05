use crate::{CommandResult, Context};

pub trait Provider<C: Context>: Default {
    type Output;
    type Error: Into<C::Error>;

    fn provide(&self, ctx: &C) -> CommandResult<C, Self::Output>;
}

pub trait Provideable<C: Context> {
    type Provider: Provider<C>;
}
