use super::{Context, Command, CommandBase, Func, Input};
use futures::{ready, TryFuture};
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{self, Poll};

#[derive(Copy, Clone, Debug)]
pub struct Exec<T, F> {
    pub(super) command: T,
    pub(super) callback: F,
}

impl<T, F, C> CommandBase<C> for Exec<T, F>
where
    C: Context,
    T: Command<C>,
    F: Func<T::Argument> + Clone + Send,
{
    type Argument = (F::Output,);
    type Future = ExecFuture<T, F, C>;

    fn parse(&self, ctx: *mut C, input: *mut Input) -> Self::Future {
        ExecFuture {
            command: self.command.parse(ctx, input),
            callback: self.callback.clone(),
        }
    }
}

#[allow(missing_debug_implementations)]
#[pin_project]
pub struct ExecFuture<T: Command<C>, F, C: Context> {
    #[pin]
    command: T::Future,
    callback: F,
}

impl<T, F, C> Future for ExecFuture<T, F, C>
where
    C: Context,
    T: Command<C>,
    F: Func<T::Argument>,
{
    type Output = Result<(F::Output,), ()>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut task::Context) -> Poll<Self::Output> {
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
