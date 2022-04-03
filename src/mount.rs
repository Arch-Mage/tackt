use tower_service::Service;

use crate::error::Error;
use crate::future::Maybe;
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
    type Param = Param;

    fn call_with_param(&mut self, req: T, _: Self::Param) -> Self::Future {
        self.call(req)
    }

    fn param(&self, req: &T) -> Result<Self::Param, Error> {
        match req.path().starts_with(self.prefix) {
            true => Ok(Param),
            false => Err(Error::Path),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Param;

impl<T> crate::param::Param<T> for Param {
    fn from_request(_: &T) -> Result<Self, Error> {
        Ok(Param)
    }
}

#[cfg(test)]
mod tests {
    use http::Request;

    use crate::error::Error;
    use crate::exec::run;
    use crate::macros::param;
    use crate::router::Router;

    param!(Root, GET, "/");
    param!(Route1, GET, "/route1");
    param!(Route2, GET, "/route2");
    param!(Other, GET, "/other");

    async fn root(_: Request<()>, _: Root) -> Result<&'static str, Error> {
        Ok("root")
    }

    async fn route1(_: Request<()>, _: Route1) -> Result<&'static str, Error> {
        Ok("route1")
    }

    async fn route2(_: Request<()>, _: Route2) -> Result<&'static str, Error> {
        Ok("route2")
    }

    async fn other(_: Request<()>, _: Other) -> Result<&'static str, Error> {
        Ok("other")
    }

    fn req(path: &'static str) -> Request<()> {
        Request::builder()
            .method(http::Method::GET)
            .uri(http::Uri::from_static(path))
            .body(())
            .unwrap()
    }

    #[test]
    fn test() {
        let root = Router::new(root).route(other);
        let r1 = Router::new(route1);
        let r2 = Router::new(route2);
        let router = root.mount("/r1", r1).mount("/r2", r2);

        let res = run(router, req("/"));
        assert_eq!(res, Ok("root"));

        let res = run(router, req("/other"));
        assert_eq!(res, Ok("other"));

        let res = run(router, req("/r1/route1"));
        assert_eq!(res, Ok("route1"));

        let res = run(router, req("/r2/route2"));
        assert_eq!(res, Ok("route2"));
    }
}
