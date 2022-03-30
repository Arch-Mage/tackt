use tower_service::Service;

use crate::error::Error;
use crate::future::Either;
use crate::future::Maybe;
use crate::param;
use crate::route::Route;

#[derive(Clone, Copy, Debug)]
pub enum Param<L, R> {
    Left(L),
    Right(R),
}

impl<L, R, T> param::Param<T> for Param<L, R>
where
    L: param::Param<T>,
    R: param::Param<T>,
{
    #[inline]
    fn from_request(req: &T) -> Result<Self, Error> {
        let err1 = match L::from_request(req) {
            Err(err) => err,
            Ok(data) => return Ok(Self::Left(data)),
        };
        let err2 = match R::from_request(req) {
            Err(err) => err,
            Ok(data) => return Ok(Self::Right(data)),
        };
        Err(std::cmp::min(err1, err2))
    }
}

/// Routing branch.
///
/// Note that application code cannot construct this struct directly. This is
/// exported for type annotation only.
#[derive(Clone, Copy, Debug)]
pub struct Or<L, R> {
    left: L,
    right: R,
}

impl<L, R> Or<L, R> {
    #[inline]
    pub(crate) fn new<T, U, E>(left: L, right: R) -> Or<L, R>
    where
        L: Route<T, Response = U, Error = E>,
        R: Route<T, Response = U, Error = E>,
        E: From<Error>,
    {
        Or { left, right }
    }
}

impl<L, R, T, U, E> Service<T> for Or<L, R>
where
    L: Route<T, Response = U, Error = E>,
    R: Route<T, Response = U, Error = E>,
    E: From<Error>,
{
    type Response = U;

    type Error = E;

    type Future = Maybe<Either<L::Future, R::Future>, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: T) -> Self::Future {
        match self.param(&req) {
            Err(err) => Maybe::ready(Err(err.into())),
            Ok(param) => self.call_with_param(req, param),
        }
    }
}

impl<L, R, T, U, E> Route<T> for Or<L, R>
where
    L: Route<T, Response = U, Error = E>,
    R: Route<T, Response = U, Error = E>,
    E: From<Error>,
{
    type Param = Param<L::Param, R::Param>;

    fn call_with_param(&mut self, req: T, param: Self::Param) -> Self::Future {
        match param {
            Param::Left(param) => {
                Maybe::Future(Either::Left(self.left.call_with_param(req, param)))
            }
            Param::Right(param) => {
                Maybe::Future(Either::Right(self.right.call_with_param(req, param)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::future::ready;

    use tower_service::Service;

    use crate::error::Error;
    use crate::exec::oneshot;
    use crate::func::Func;
    use crate::or::Or;
    use crate::param::Param;

    #[test]
    fn test() {
        struct Left(i32);
        struct Right(i32);

        impl Param<&'static str> for Left {
            fn from_request(req: &&'static str) -> Result<Self, Error> {
                req.strip_prefix("/left/")
                    .ok_or(Error::Path)?
                    .parse()
                    .map(Left)
                    .map_err(|_| Error::Path)
            }
        }

        impl Param<&'static str> for Right {
            fn from_request(req: &&'static str) -> Result<Self, Error> {
                req.strip_prefix("/right/")
                    .ok_or(Error::Path)?
                    .parse()
                    .map(Right)
                    .map_err(|_| Error::Path)
            }
        }

        let mut or = Or::new(
            Func::new(|_: &str, param: Left| ready(Ok::<_, Error>(param.0))),
            Func::new(|_: &str, param: Right| ready(Ok::<_, Error>(param.0))),
        );

        let res = oneshot(or.call("/"));
        assert!(matches!(res, Err(Error::Path)));
        let res = oneshot(or.call("/left"));
        assert!(matches!(res, Err(Error::Path)));
        let res = oneshot(or.call("/right"));
        assert!(matches!(res, Err(Error::Path)));
        let res = oneshot(or.call("/left/1"));
        assert!(matches!(res, Ok(1)));
        let res = oneshot(or.call("/right/2"));
        assert!(matches!(res, Ok(2)));
    }
}
