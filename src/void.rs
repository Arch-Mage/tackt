use std::future::Ready;
use std::marker::PhantomData;

use tower_service::Service;

use crate::error::Error;
use crate::route::Route;

/// Void, nothing.
///
/// No one will find anything there.
pub struct Void<T, U>(PhantomData<T>, PhantomData<U>);

impl<T, U> std::fmt::Debug for Void<T, U> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Void").finish()
    }
}

impl<T, U> Void<T, U> {
    pub(crate) const fn new() -> Void<T, U> {
        Void(PhantomData, PhantomData)
    }
}

impl<T, U> Copy for Void<T, U> {}

impl<T, U> Clone for Void<T, U> {
    fn clone(&self) -> Self {
        Void(self.0, self.1)
    }
}

impl<T, U> Service<T> for Void<T, U> {
    type Response = U;

    type Error = Error;

    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _: &mut std::task::Context,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        std::future::ready(Err(Error::Path))
    }
}

impl<T, U> Route<T> for Void<T, U> {
    type Param = Param;

    fn call_with_param(&mut self, _: T, _: Self::Param) -> Self::Future {
        std::future::ready(Err(Error::Path))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Param;

impl<T> crate::param::Param<T> for Param {
    fn from_request(_: &T) -> Result<Self, Error> {
        Err(Error::Path)
    }
}
