use super::{Input, Command, CommandBase, Func};
use pin_project::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};
use futures::{ready, TryFuture};
use std::future::Future;

#[derive(Copy, Clone, Debug)]
pub struct Exec<T, F> {
    pub(super) command: T,
    pub(super) callback: F,
}

impl<T, F> CommandBase for Exec<T, F>
where
    T: Command,
    F: Func<T::Argument> + Clone + Send,
{
    type Argument = (F::Output,);
    type Future = ExecFuture<T, F>;

    fn parse(&self, input: *mut Input) -> Self::Future {
        ExecFuture {
            command: self.command.parse(input),
            callback: self.callback.clone(),
        }
    }
}

#[allow(missing_debug_implementations)]
#[pin_project]
pub struct ExecFuture<T: Command, F> {
    #[pin]
    command: T::Future,
    callback: F,
}

impl<T, F> Future for ExecFuture<T, F>
where
    T: Command,
    F: Func<T::Argument>,
{
    type Output = Result<(F::Output,), ()>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let pin = self.project();
        match ready!(pin.command.try_poll(cx)) {
            Ok(ex) => {
                let ex = (pin.callback.call(ex),);
                Poll::Ready(Ok(ex))
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}