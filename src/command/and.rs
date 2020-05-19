use super::{Context, Combine, Command, CommandBase, HList, Input, Tuple};
use futures::ready;
use pin_project::{pin_project, project};
use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::task::{self, Poll};

#[derive(Debug, Copy, Clone)]
pub struct And<T, U> {
    pub(super) first: T,
    pub(super) second: U,
}

impl<T, U, C: Context> CommandBase<C> for And<T, U>
where
    T: Command<C>,
    T::Argument: Send,
    U: Command<C> + Clone + Send,
    <T::Argument as Tuple>::HList: Combine<<U::Argument as Tuple>::HList> + Send,
{
    type Argument = <<<T::Argument as Tuple>::HList as Combine<<U::Argument as Tuple>::HList>>::Output as HList>::Tuple;
    type Future = AndFuture<T, U, C>;

    fn parse(&self, ctx: *mut C, input: *mut Input) -> Self::Future {
        let (ctx, input,) = unsafe { (&mut *ctx, &mut *input,) };
        AndFuture {
            state: State::First(self.first.parse(ctx, input), self.second.clone()),
            input: unsafe { mem::transmute::<*mut Input<'_>, *mut Input<'static>>(input) },
            context: ctx,
        }
    }
}

#[allow(missing_debug_implementations)]
#[pin_project]
pub struct AndFuture<T: Command<C>, U: Command<C>, C: Context> {
    #[pin]
    state: State<T, U, C>,
    input: *mut Input<'static>,
    context: *mut C,
}

#[pin_project]
enum State<T: Command<C>, U: Command<C>, C: Context> {
    First(#[pin] T::Future, U),
    Second(Option<T::Argument>, #[pin] U::Future),
    Done,
}

impl<T, U, C> Future for AndFuture<T, U, C>
where
    C: Context,
    T: Command<C>,
    U: Command<C>,
    <T::Argument as Tuple>::HList: Combine<<U::Argument as Tuple>::HList> + Send,
{
    type Output = Result<
            <<<T::Argument as Tuple>::HList as Combine<<U::Argument as Tuple>::HList>>::Output as HList>::Tuple,
            ()>;

    #[project]
    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context) -> Poll<Self::Output> {
        let (ctx, input) = unsafe { (&mut *self.context, &mut *self.input,) };
        loop {
            let pin = self.as_mut().project();
            #[project]
            let (ex1, fut2) = match pin.state.project() {
                State::First(first, second) => match ready!(first.poll(cx)) {
                    Ok(first) => (first, second.parse(ctx, input)),
                    Err(err) => return Poll::Ready(Err(From::from(err))),
                },
                State::Second(ex1, second) => {
                    let ex2 = match ready!(second.poll(cx)) {
                        Ok(second) => second,
                        Err(err) => return Poll::Ready(Err(From::from(err))),
                    };
                    let ex3 = ex1.take().unwrap().hlist().combine(ex2.hlist()).flatten();
                    self.set(AndFuture {
                        state: State::Done,
                        input,
                        context: ctx,
                    });
                    return Poll::Ready(Ok(ex3));
                }
                State::Done => panic!("polled after complete"),
            };

            self.set(AndFuture {
                state: State::Second(Some(ex1), fut2),
                input,
                context: ctx,
            });
        }
    }
}
