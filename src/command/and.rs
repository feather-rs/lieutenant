use super::{Input, Command, CommandBase, Combine, Tuple, HList};
use pin_project::{pin_project, project};
use std::pin::Pin;
use std::future::Future;
use std::task::{self, Poll};
use futures::ready;
use std::mem;

#[derive(Debug, Copy, Clone)]
pub struct And<T, U> {
    pub(super) first: T,
    pub(super) second: U,
}

impl<T, U> CommandBase for And<T, U>
where
    T: Command,
    T::Argument: Send,
    U: Command + Clone + Send,
    <T::Argument as Tuple>::HList: Combine<<U::Argument as Tuple>::HList> + Send,
{
    type Argument = <<<T::Argument as Tuple>::HList as Combine<<U::Argument as Tuple>::HList>>::Output as HList>::Tuple;
    type Future = AndFuture<T, U>;

    fn parse(&self, input: *mut Input) -> Self::Future {
        let (input,) = unsafe { (&mut *input,) };
        AndFuture {
            state: State::First(self.first.parse(input), self.second.clone()),
            input: unsafe { mem::transmute::<*mut Input<'_>, *mut Input<'static>>(input) },
        }
    }   
}

#[allow(missing_debug_implementations)]
#[pin_project]
pub struct AndFuture<T: Command, U: Command> {
    #[pin]
    state: State<T, U>,
    input: *mut Input<'static>,
}

#[pin_project]
enum State<T: Command, U: Command> {
    First(#[pin] T::Future, U),
    Second(Option<T::Argument>, #[pin] U::Future),
    Done,
}

impl<T, U> Future for AndFuture<T, U>
where
    T: Command,
    U: Command,
    <T::Argument as Tuple>::HList: Combine<<U::Argument as Tuple>::HList> + Send,
{
    type Output = Result<
            <<<T::Argument as Tuple>::HList as Combine<<U::Argument as Tuple>::HList>>::Output as HList>::Tuple,
            ()>;

    #[project]
    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context) -> Poll<Self::Output> {
        let (input,) = unsafe { (&mut *self.input,) };
        loop {
            let pin = self.as_mut().project();
            #[project]
            let (ex1, fut2) = match pin.state.project() {
                State::First(first, second) => match ready!(first.poll(cx)) {
                    Ok(first) => (first, second.parse(input)),
                    Err(err) => return Poll::Ready(Err(From::from(err))),
                },
                State::Second(ex1, second) => {
                    let ex2 = match ready!(second.poll(cx)) {
                        Ok(second) => second,
                        Err(err) => return Poll::Ready(Err(From::from(err))),
                    };
                    let ex3 = ex1.take().unwrap().hlist().combine(ex2.hlist()).flatten();
                    self.set(AndFuture { state: State::Done, input });
                    return Poll::Ready(Ok(ex3));
                }
                State::Done => panic!("polled after complete"),
            };

            self.set(AndFuture {
                state: State::Second(Some(ex1), fut2),
                input: input,
            });
        }
    }
}