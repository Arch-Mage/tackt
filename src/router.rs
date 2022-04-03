use std::future::Future;

use tower_service::Service;

use crate::error::Error;
use crate::func::Func;
use crate::mount::Mount;
use crate::or::Or;
use crate::param::Param;
use crate::request::PathReq;
use crate::request::RemovePrefix;
use crate::route::Route;
use crate::void::Void;
use crate::with::With;

/// The router instance.
///
/// Note that a router does not implement Route. It only implement service.
#[derive(Clone, Copy, Debug)]
pub struct Router<R> {
    inner: R,
}

impl<R, T> Service<T> for Router<R>
where
    R: Service<T>,
{
    type Response = R::Response;

    type Error = R::Error;

    type Future = R::Future;

    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: T) -> Self::Future {
        self.inner.call(req)
    }
}

impl<T, U> Router<Void<T, U>> {
    /// Create a router that does not match any request.
    pub const fn void() -> Router<Void<T, U>> {
        Router { inner: Void::new() }
    }
}

impl<F, P> Router<Func<F, P>> {
    /// Create a new router with the route.
    #[inline]
    pub fn new<T, U, E, Fut>(route: F) -> Router<Func<F, P>>
    where
        F: FnMut(T, P) -> Fut,
        P: Param<T>,
        E: From<Error>,
        Fut: Future<Output = Result<U, E>>,
    {
        Router {
            inner: Func::new(route),
        }
    }
}

impl<R> Router<R> {
    /// Add new route to this router.
    #[inline]
    pub fn route<F, T, P, U, E, Fut>(self, route: F) -> Router<Or<R, Func<F, P>>>
    where
        R: Route<T, Response = U, Error = E>,
        F: FnMut(T, P) -> Fut,
        P: Param<T>,
        E: From<Error>,
        Fut: Future<Output = Result<U, E>>,
    {
        Router {
            inner: Or::new(self.inner, Func::new(route)),
        }
    }

    /// Mount a service at prefix.
    ///
    /// Any request to prefix will be delegated to the service with the prefix
    /// stripped.
    ///
    /// *NOTE*: `prefix` will be stripped from any trailing slash.
    ///
    /// # Panic
    ///
    /// Panic if one of these conditions is met:
    ///
    /// 1. Prefix contains `?`.
    /// 2. Prefix contains `#`.
    /// 3. Prefix is empty.
    /// 4. Prefix is `/`.
    /// 5. Prefix does not start with `/`.
    #[inline]
    pub fn mount<S, T, U, E>(self, prefix: &'static str, service: S) -> Router<Or<R, Mount<S>>>
    where
        R: Route<T, Response = U, Error = E>,
        S: Service<T, Response = U, Error = E>,
        T: PathReq + RemovePrefix,
        E: From<Error>,
    {
        let prefix = prefix.trim_end_matches('/');
        assert!(!prefix.contains('?'), "Prefix cannot contains '?'");
        assert!(!prefix.contains('#'), "Prefix cannot contains '#'");
        assert!(!prefix.is_empty(), "Prefix cannot be empty");
        assert!(prefix != "/", "Prefix cannot be '/'");
        assert!(prefix.starts_with('/'), "Prefix must be starts with '/'");
        Router {
            inner: Or::new(self.inner, Mount::new(service, prefix)),
        }
    }

    /// Add middleware to the router.
    #[inline]
    pub fn with<F, T, Fut>(self, func: F) -> Router<With<R, F>>
    where
        R: Clone + Service<T>,
        F: FnMut(T) -> Fut,
        Fut: Future<Output = Result<T, R::Error>>,
    {
        Router {
            inner: With::new(self.inner, func),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;
    use std::net::SocketAddr;

    use http::Request;
    use http::Response;
    use hyper::service::make_service_fn;
    use hyper::Body;

    use crate::error::Error;
    use crate::param::Param;
    use crate::Router;

    #[derive(Clone, Copy, Debug)]
    struct Home;

    impl Param<Request<Body>> for Home {
        fn from_request(req: &Request<Body>) -> Result<Self, Error> {
            match req.uri().path() {
                "/" => Ok(Home),
                _ => Err(Error::Path),
            }
        }
    }

    async fn home(req: Request<Body>, param: Home) -> Result<Response<Body>, Error> {
        Ok(Response::new(Body::from(format!(
            "{:?} @ {}",
            param,
            req.uri().path()
        ))))
    }

    #[derive(Clone, Copy, Debug)]
    struct About;

    impl Param<Request<Body>> for About {
        fn from_request(req: &Request<Body>) -> Result<Self, Error> {
            match req.uri().path() {
                "/about" => Ok(About),
                _ => Err(Error::Path),
            }
        }
    }

    async fn about(req: Request<Body>, param: About) -> Result<Response<Body>, Error> {
        Ok(Response::new(Body::from(format!(
            "{:?} @ {}",
            param,
            req.uri().path()
        ))))
    }

    #[test]
    #[allow(dead_code)]
    fn compile() {
        let subrouter = Router::new(home).route(about);
        let router = Router::new(home).route(about).mount("/sub", subrouter);

        let _ = |addr: SocketAddr| async move {
            let _ = hyper::server::Server::bind(&addr)
                .serve(make_service_fn(
                    |_| async move { Ok::<_, Infallible>(router) },
                ))
                .await;
        };
    }
}
