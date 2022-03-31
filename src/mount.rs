use tower_service::Service;

use crate::error::Error;
use crate::future::Maybe;
use crate::param::Through;
use crate::request::PathReq;
use crate::request::RemovePrefix;
use crate::route::Route;

/// Mount a Service.
///
/// Note that application code cannot construct this struct directly. This is
/// exported for type annotation only.
#[derive(Clone, Copy, Debug)]
pub struct Mount<S> {
    inner: S,
    prefix: &'static str,
}

impl<S> Mount<S> {
    #[inline]
    pub(crate) fn new<T>(inner: S, prefix: &'static str) -> Mount<S>
    where
        S: Service<T>,
        T: PathReq + RemovePrefix,
        S::Error: From<Error>,
    {
        Mount { inner, prefix }
    }
}

impl<S, T> Service<T> for Mount<S>
where
    S: Service<T>,
    T: PathReq + RemovePrefix,
    S::Error: From<Error>,
{
    type Response = S::Response;

    type Error = S::Error;

    type Future = Maybe<S::Future, Result<Self::Response, Self::Error>>;

    #[inline]
    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: T) -> Self::Future {
        if self.prefix.contains('?') {
            return Maybe::ready(Err(Error::Prefix.into()));
        };
        match req.remove_prefix(self.prefix) {
            Err(err) => Maybe::ready(Err(err.into())),
            Ok(req) => Maybe::Future(self.inner.call(req)),
        }
    }
}

impl<S, T> Route<T> for Mount<S>
where
    S: Service<T>,
    T: PathReq + RemovePrefix,
    S::Error: From<Error>,
{
    type Param = Through;

    fn call_with_param(&mut self, req: T, _: Self::Param) -> Self::Future {
        self.call(req)
    }
}
