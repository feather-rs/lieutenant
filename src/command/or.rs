use super::{Input, Command, CommandBase};
use crate::generic::Either;
use pin_project::{pin_project, project};
use std::pin::Pin;
use std::future::Future;
use std::task::{self, Poll};
use futures::{ready, TryFuture};
use std::mem;

#[derive(Clone, Copy, Debug)]
pub struct Or<T, U> {
    pub(super) first: T,
    pub(super) second: U,
}

impl<T, U> CommandBase for Or<T, U>
where
    T: Command,
    U: Command + Send + Clone,
{
    type Argument = (Either<T::Argument, U::Argument>,);
    type Future = EitherFuture<T, U>;
    
    fn parse(&self, input: *mut Input) -> Self::Future {
        EitherFuture {
            state: State::First(self.first.parse(input.clone()), self.second.clone()),
            input: unsafe { mem::transmute::<*mut Input<'_>, *mut Input<'static>>(input) },
        }
    }
}

#[allow(missing_debug_implementations)]
#[pin_project]
pub struct EitherFuture<T: Command, U: Command> {
    #[pin]
    state: State<T, U>,
    input: *mut Input<'static>,
}

#[pin_project]
enum State<T: Command, U: Command> {
    First(#[pin] T::Future, U),
    Second(Option<()>, #[pin] U::Future),
    Done,
}

impl<T, U> Future for EitherFuture<T, U>
where
    T: Command,
    U: Command,
{
    type Output = Result<(Either<T::Argument, U::Argument>,), ()>;

    #[project]
    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context) -> Poll<Self::Output> {
        let input = &self.input.clone();
        loop {
            let pin = self.as_mut().project();
            #[project]
            let (err1, fut2) = match pin.state.project() {
                State::First(first, second) => match ready!(first.try_poll(cx)) {
                    Ok(ex1) => {
                        return Poll::Ready(Ok((Either::A(ex1),)));
                    }
                    Err(e) => {
                        (e, second.parse(input.clone()))
                    }
                },
                State::Second(_, second) => {
                    let ex2 = match ready!(second.try_poll(cx)) {
                        Ok(ex2) => Ok((Either::B(ex2),)),
                        Err(e) => {
                            Err(e)
                        }
                    };
                    self.set(EitherFuture {
                        state: State::Done,
                        input: input.clone(),
                        ..*self
                    });
                    return Poll::Ready(ex2);
                }
                State::Done => panic!("polled after complete"),
            };

            self.set(EitherFuture {
                state: State::Second(Some(err1), fut2),
                input: input.clone(),
                ..*self
            });
        }
    }
}