use std::future::Future;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

#[derive(Clone, Copy, Debug)]
#[pin_project::pin_project(project = ProjectedMaybe)]
pub enum Maybe<F, O> {
    Future(#[pin] F),
    Ready(Option<O>),
}

impl<F, O> Maybe<F, O> {
    #[inline]
    pub const fn ready(value: O) -> Maybe<F, O> {
        Maybe::Ready(Some(value))
    }
}

impl<F, O> Future for Maybe<F, O>
where
    F: Future<Output = O>,
{
    type Output = O;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project() {
            ProjectedMaybe::Future(fut) => Future::poll(fut, cx),
            ProjectedMaybe::Ready(out) => match out.take() {
                None => unreachable!("polled after ready"),
                Some(out) => Poll::Ready(out),
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[pin_project::pin_project(project = ProjectedEither)]
pub enum Either<L, R> {
    Left(#[pin] L),
    Right(#[pin] R),
}

impl<L, R, O> Future for Either<L, R>
where
    L: Future<Output = O>,
    R: Future<Output = O>,
{
    type Output = O;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project() {
            ProjectedEither::Left(fut) => Future::poll(fut, cx),
            ProjectedEither::Right(fut) => Future::poll(fut, cx),
        }
    }
}
