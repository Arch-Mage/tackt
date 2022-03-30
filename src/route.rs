use tower_service::Service;

use crate::error::Error;
use crate::param::Param;

/// A route is a [`Service`][1] that has a [`Param`] to determine wether the
/// service should be called or not.
///
/// [1]: https://docs.rs/tower-service/0.3/tower_service/trait.Service.html
pub trait Route<T>: Service<T> {
    /// The associated param to determine wether this route is match or not.
    type Param: Param<T>;

    /// Implementors should call this after obtaining `Param` in `Service`'s
    /// call.
    fn call_with_param(&mut self, req: T, param: Self::Param) -> Self::Future;

    /// A helper method to obtain `Param` in `Service`'s call.
    ///
    /// Without this, you'll have to write something like:
    ///
    /// ```ignored
    /// <Self as Route<T>>::Param::from_request(&req);
    /// ```
    #[inline]
    fn param(&self, req: &T) -> Result<Self::Param, Error> {
        Self::Param::from_request(req)
    }
}
