use std::convert::Infallible;
use std::future::Ready;
use std::marker::PhantomData;

use hyper::server::conn::AddrStream;
use tower_service::Service;

#[derive(Debug)]
pub struct Make<S, T> {
    inner: S,

    request: PhantomData<T>,
}

impl<S, T> Make<S, T> {
    #[inline]
    pub const fn new(inner: S) -> Make<S, T> {
        Make {
            inner,
            request: PhantomData,
        }
    }
}

impl<S: Copy, T> Copy for Make<S, T> {}

impl<S: Clone, T> Clone for Make<S, T> {
    fn clone(&self) -> Self {
        Make {
            inner: self.inner.clone(),
            request: self.request,
        }
    }
}

impl<'a, T, S> Service<&'a AddrStream> for Make<S, T>
where
    S: Clone + Service<T>,
{
    type Response = S;

    type Error = Infallible;

    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: &'a AddrStream) -> Self::Future {
        std::future::ready(Ok(self.inner.clone()))
    }
}
