use std::future::Future;
use std::marker::PhantomData;

use tower_service::Service;

use crate::error::Error;
use crate::future::Maybe;
use crate::param::Param;
use crate::route::Route;

/// Wrap a function into a route.
///
/// Note that application code cannot construct this struct directly. This is
/// exported for type annotation only.
#[derive(Debug)]
pub struct Func<F, P> {
    inner: F,

    param: PhantomData<P>,
}

impl<F, P> Func<F, P> {
    #[inline]
    pub(crate) fn new<T, U, E, Fut>(inner: F) -> Func<F, P>
    where
        F: FnMut(T, P) -> Fut,
        P: Param<T>,
        E: From<Error>,
        Fut: Future<Output = Result<U, E>>,
    {
        Func {
            inner,
            param: PhantomData,
        }
    }
}

impl<F: Copy, P> Copy for Func<F, P> {}

impl<F: Clone, P> Clone for Func<F, P> {
    fn clone(&self) -> Self {
        Func {
            inner: self.inner.clone(),
            param: self.param,
        }
    }
}

impl<F, T, P, U, E, Fut> Service<T> for Func<F, P>
where
    F: FnMut(T, P) -> Fut,
    P: Param<T>,
    E: From<Error>,
    Fut: Future<Output = Result<U, E>>,
{
    type Response = U;

    type Error = E;

    type Future = Maybe<Fut, Result<U, E>>;

    #[inline]
    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, req: T) -> Self::Future {
        match self.param(&req) {
            Ok(param) => self.call_with_param(req, param),
            Err(err) => Maybe::ready(Err(err.into())),
        }
    }
}

impl<F, T, P, U, E, Fut> Route<T> for Func<F, P>
where
    F: FnMut(T, P) -> Fut,
    P: Param<T>,
    E: From<Error>,
    Fut: Future<Output = Result<U, E>>,
{
    type Param = P;

    #[inline]
    fn call_with_param(&mut self, req: T, param: Self::Param) -> Self::Future {
        Maybe::Future((self.inner)(req, param))
    }
}

#[cfg(test)]
mod tests {
    use super::Error;
    use super::Func;

    #[test]
    fn test() {
        #[derive(Debug)]
        struct Param;

        impl crate::param::Param<&'static str> for Param {
            fn from_request(req: &&'static str) -> Result<Self, Error> {
                if *req != "/" {
                    return Err(Error::Path);
                }
                Ok(Param)
            }
        }

        let func = Func::new(|_: &'static str, param: Param| async { Ok::<_, Error>(param) });
        let res = crate::exec::run(func, "/");
        assert!(matches!(res, Ok(Param)));
        let res = crate::exec::run(func, "/somewhere");
        assert!(matches!(res, Err(Error::Path)));
    }
}
