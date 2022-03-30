use http::uri::PathAndQuery;
use http::Request;
use http::Uri;

use crate::error::Error;

/// A request that has a path.
///
/// This trait is required when using [`#[derive(Param)]`][1] or [`#[route]`][2]
/// attribute.
///
/// [1]: crate::param::Param
/// [2]: macro@crate::route
pub trait PathReq {
    /// The associated path.
    fn path(&self) -> &str;
}

impl PathReq for &str {
    #[inline]
    fn path(&self) -> &str {
        self
    }
}

impl PathReq for String {
    #[inline]
    fn path(&self) -> &str {
        self.as_str()
    }
}

impl PathReq for PathAndQuery {
    #[inline]
    fn path(&self) -> &str {
        self.as_str()
    }
}

impl PathReq for Uri {
    #[inline]
    fn path(&self) -> &str {
        Uri::path(self)
    }
}

impl<T> PathReq for Request<T> {
    #[inline]
    fn path(&self) -> &str {
        self.uri().path()
    }
}

/// A request that has an HTTP method.
///
/// This trait is required when using [`#[derive(Param)]`][1] or [`#[route]`][2]
/// attribute.
///
/// [1]: crate::param::Param
/// [2]: tackt_macros::route
pub trait MethodReq {
    /// The associated method.
    fn method(&self) -> &http::Method;
}

impl<T> MethodReq for Request<T> {
    #[inline]
    fn method(&self) -> &http::Method {
        http::Request::method(self)
    }
}

/// A request that can remove it's prefix.
///
/// This trait is required by [`Mount`][1].
///
/// [1]: crate::mount::Mount
pub trait RemovePrefix: Sized {
    /// This function should returns [`Error::Prefix`][1] when path does not
    /// start with `prefix` or the `prefix` itself is invalid.
    ///
    /// `prefix` is considered invalid when it fails to be parsed as
    /// [`PathAndQuery`][2] or it contains '?' or '#'.
    ///
    /// [1]: crate::error::Error::Prefix
    /// [2]: http::uri::PathAndQuery
    fn remove_prefix(self, prefix: &str) -> Result<Self, Error>;
}

impl RemovePrefix for String {
    fn remove_prefix(self, prefix: &str) -> Result<String, Error> {
        if prefix.contains('?') {
            return Err(Error::Prefix);
        };
        self.as_str()
            .strip_prefix(prefix)
            .map(String::from)
            .ok_or(Error::Prefix)
    }
}

impl RemovePrefix for PathAndQuery {
    fn remove_prefix(self, prefix: &str) -> Result<PathAndQuery, Error> {
        if prefix.contains('?') {
            return Err(Error::Prefix);
        };

        self.as_str()
            .strip_prefix(prefix)
            .and_then(|striped| striped.parse().ok())
            .ok_or(Error::Prefix)
    }
}

impl RemovePrefix for Uri {
    fn remove_prefix(self, prefix: &str) -> Result<Uri, Error> {
        if prefix.contains('?') {
            return Err(Error::Prefix);
        }
        let mut parts = self.into_parts();
        parts.path_and_query = match parts.path_and_query.take() {
            None => return Err(Error::Prefix),
            Some(p_and_q) => Some(p_and_q.remove_prefix(prefix)?),
        };
        Uri::from_parts(parts).map_err(|_| Error::Prefix)
    }
}

impl<T> RemovePrefix for Request<T> {
    #[inline]
    fn remove_prefix(self, prefix: &str) -> Result<Request<T>, Error> {
        if prefix.contains('?') {
            return Err(Error::Prefix);
        }
        let (mut parts, body) = self.into_parts();
        parts.uri = parts.uri.remove_prefix(prefix)?;
        Ok(Request::from_parts(parts, body))
    }
}

#[test]
fn test() {
    let val = "/a/b".to_string();
    let val = val.remove_prefix("/a").unwrap();
    assert_eq!(val, "/b");

    let val = PathAndQuery::from_static("/a/b");
    let val = val.remove_prefix("/a").unwrap();
    assert_eq!(val, "/b");

    let val = Uri::from_static("/a/b");
    let val = val.remove_prefix("/a").unwrap();
    assert_eq!(val, "/b");

    let val = Request::<()>::builder().uri("/a/b").body(()).unwrap();
    let val = val.remove_prefix("/a").unwrap();
    assert_eq!(val.uri(), "/b");
}
