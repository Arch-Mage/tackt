use crate::error::Error;

/// A param describes a [route][1]'s dependency ([route][1]'s second argument).
///
/// `T` is the request type. With [`hyper`][2], it will be [`http::Request`][3]
///
/// When using `#[derive(Param)]`, `T` must implements both [`PathReq`][4] and
/// [`MethodReq`][5].
///
/// [1]: crate::route::Route
/// [2]: https://docs.rs/hyper/0.14
/// [3]: https://docs.rs/http/0.2/http/request/struct.Request.html
/// [4]: crate::request::PathReq
/// [5]: crate::request::MethodReq
pub trait Param<T>: Sized {
    /// Construct param from request.
    ///
    /// [`Error::Path`][1] should be returned when request's path does not match.
    ///
    /// [`Error::Method`][2] should be returned when request's method does not
    /// match.
    ///
    /// [`Error::Prefix`][3] should not be returned. It's used specifically by
    /// [`mount`][4]
    ///
    /// [1]: crate::error::Error::Path
    /// [2]: crate::error::Error::Method
    /// [3]: crate::error::Error::Prefix
    /// [4]: crate::router::Router::mount
    fn from_request(req: &T) -> Result<Self, Error>;
}
