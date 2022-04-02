/// Build Router from several routes.
///
/// This is just a shortcut for constructing something like this.
///
/// ```
/// # use tackt::Error;
/// # use tackt::Router;
/// # use tackt::routes;
/// #
/// # struct Param;
/// # impl<T> tackt::Param<T> for Param {
/// #     fn from_request(req: &T) -> Result<Self, Error> {
/// #         Ok(Param)
/// #     }
/// # }
/// # async fn route1(req: String, param: Param) -> Result<String, Error> { Ok(req) }
/// # async fn route2(req: String, param: Param) -> Result<String, Error> { Ok(req) }
/// # async fn route3(req: String, param: Param) -> Result<String, Error> { Ok(req) }
/// let router = Router::new(route1)
///     .route(route2)
///     .route(route3);
///
/// let router = routes![route3, route2, route1];
/// ```
///
/// **NOTE** the route(s) is assigned backward.
#[macro_export]
macro_rules! routes {
    ($route:expr $(,)?) => {
        $crate::Router::new($route)
    };

    ($route:expr, $($rest:expr),+ $(,)?) => {
        $crate::routes!($($rest),+).route($route)
    }
}

/// Generate an empty struct that implements [`Param`][crate::param::Param].
#[cfg(test)]
macro_rules! param {
    ($Type:ident, $method:ident, $path:expr) => {
        struct $Type;

        impl<T> $crate::param::Param<::http::request::Request<T>> for $Type {
            #[inline]
            fn from_request(
                req: &::http::request::Request<T>,
            ) -> Result<Self, $crate::error::Error> {
                if req.method() != &::http::Method::$method {
                    return Err($crate::error::Error::Method);
                }

                if req.uri().path() != $path {
                    return Err($crate::error::Error::Path);
                }

                Ok($Type)
            }
        }
    };
    ($Type:ident, $method:ident) => {
        struct $Type;

        impl<T> $crate::param::Param<::http::request::Request<T>> for $Type {
            #[inline]
            fn from_request(
                req: &::http::request::Request<T>,
            ) -> Result<Self, $crate::error::Error> {
                if req.uri().path() != $path {
                    return Err($crate::error::Error::Path);
                }

                Ok($Type)
            }
        }
    };
    ($Type:ident, $path:expr) => {
        struct $Type;

        impl<T> $crate::param::Param<::http::request::Request<T>> for $Type {
            #[inline]
            fn from_request(
                req: &::http::request::Request<T>,
            ) -> Result<Self, $crate::error::Error> {
                if req.uri().path() != $path {
                    return Err($crate::error::Error::Path);
                }

                Ok($Type)
            }
        }
    };
}

#[cfg(test)]
pub(crate) use param;
