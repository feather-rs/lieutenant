use crate::Context;

pub trait Provider<C: Context> {
    type Output;
    type Error: Into<C::Error>;

    fn provide(&self, ctx: &C) -> Result<Self::Output, Self::Error>;
}

pub trait Provideable<C: Context> {
    type Provider: Provider<C>;
}
