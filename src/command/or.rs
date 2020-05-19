use super::{Context, Command, CommandBase, Input};
use crate::generic::Either;
use futures::{ready, TryFuture};
use pin_project::{pin_project, project};
use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::task::{self, Poll};

#[derive(Clone, Copy, Debug)]
pub struct Or<T, U> {
    pub(super) first: T,
    pub(super) second: U,
}

impl<T, U, C> CommandBase<C> for Or<T, U>
where
    C: Context,
    T: Command<C>,
    U: Command<C> + Send + Clone,
{
    type Argument = (Either<T::Argument, U::Argument>,);
    type Future = EitherFuture<T, U, C>;

    fn parse(&self, ctx: *mut C, input: *mut Input) -> Self::Future {
        EitherFuture {
            state: State::First(self.first.parse(ctx, input.clone()), self.second.clone()),
            input: unsafe { mem::transmute::<*mut Input<'_>, *mut Input<'static>>(input) },
            context: ctx,
        }
    }
}

#[allow(missing_debug_implementations)]
#[pin_project]
pub struct EitherFuture<T: Command<C>, U: Command<C>, C: Context> {
    #[pin]
    state: State<T, U, C>,
    input: *mut Input<'static>,
    context: *mut C,
}

#[pin_project]
enum State<T: Command<C>, U: Command<C>, C: Context> {
    First(#[pin] T::Future, U),
    Second(Option<()>, #[pin] U::Future),
    Done,
}

impl<T, U, C> Future for EitherFuture<T, U, C>
where
    C: Context,
    T: Command<C>,
    U: Command<C>,
{
    type Output = Result<(Either<T::Argument, U::Argument>,), ()>;

    #[project]
    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context) -> Poll<Self::Output> {
        let input = &self.input.clone();
        let ctx = &self.context.clone();
        loop {
            let pin = self.as_mut().project();
            #[project]
            let (err1, fut2) = match pin.state.project() {
                State::First(first, second) => match ready!(first.try_poll(cx)) {
                    Ok(ex1) => {
                        return Poll::Ready(Ok((Either::A(ex1),)));
                    }
                    Err(e) => (e, second.parse(*ctx, *input)),
                },
                State::Second(_, second) => {
                    let ex2 = match ready!(second.try_poll(cx)) {
                        Ok(ex2) => Ok((Either::B(ex2),)),
                        Err(e) => Err(e),
                    };
                    self.set(EitherFuture {
                        state: State::Done,
                        input: *input,
                        ..*self
                    });
                    return Poll::Ready(ex2);
                }
                State::Done => panic!("polled after complete"),
            };

            self.set(EitherFuture {
                state: State::Second(Some(err1), fut2),
                input: *input,
                ..*self
            });
        }
    }
}
