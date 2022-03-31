use std::future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use tower_service::Service;

#[derive(Clone, Copy, Debug)]
pub struct With<S, F> {
    inner: S,
    func: F,
}

impl<S, F> With<S, F> {
    pub(crate) fn new<T, Fut>(inner: S, func: F) -> With<S, F>
    where
        S: Clone + Service<T>,
        F: FnMut(T) -> Fut,
        Fut: future::Future<Output = Result<T, S::Error>>,
    {
        With { inner, func }
    }
}

impl<T, S, F, Fut> Service<T> for With<S, F>
where
    S: Clone + Service<T>,
    F: FnMut(T) -> Fut,
    Fut: future::Future<Output = Result<T, S::Error>>,
{
    type Response = S::Response;

    type Error = S::Error;

    type Future = Future<Fut, S, T>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[allow(unused_must_use)]
    #[allow(clippy::todo)]
    fn call(&mut self, req: T) -> Self::Future {
        Future::new((self.func)(req), self.inner.clone())
    }
}

#[derive(Debug)]
#[pin_project::pin_project]
pub struct Future<F, S, T> {
    #[pin]
    fut: F,
    svc: S,

    req: PhantomData<T>,
}

impl<F, S, T> Future<F, S, T> {
    fn new(fut: F, svc: S) -> Future<F, S, T> {
        Future {
            fut,
            svc,
            req: PhantomData,
        }
    }
}

impl<F: Copy, S: Copy, T> Copy for Future<F, S, T> {}

impl<F: Clone, S: Clone, T> Clone for Future<F, S, T> {
    fn clone(&self) -> Self {
        Future {
            fut: self.fut.clone(),
            svc: self.svc.clone(),
            req: self.req,
        }
    }
}

impl<F, S, T, U, E> future::Future for Future<F, S, T>
where
    S: Service<T, Error = E, Response = U>,
    F: future::Future<Output = Result<T, E>>,
{
    type Output = Result<U, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.fut.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(out) => match out {
                Err(err) => Poll::Ready(Err(err)),
                Ok(req) => {
                    // let mut res = this.svc.call(req);
                    // let mut res = unsafe { Pin::new_unchecked(&mut res) };
                    let mut res = Box::pin(this.svc.call(req));
                    res.as_mut().poll(cx)
                }
            },
        }
    }
}
