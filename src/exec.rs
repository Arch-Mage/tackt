use std::future::Future;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;
use std::task::Wake;

use tower_service::Service;

pub(crate) struct Waker(std::thread::Thread);

impl Waker {
    pub(crate) fn new() -> Arc<Waker> {
        Arc::new(Waker(std::thread::current()))
    }
}

impl Wake for Waker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

/// Execute an async.
pub(crate) fn oneshot<T>(fut: impl Future<Output = T>) -> T {
    let mut fut = Box::pin(fut);
    let waker = Waker::new().into();
    let mut cx = Context::from_waker(&waker);

    match fut.as_mut().poll(&mut cx) {
        Poll::Ready(out) => out,
        Poll::Pending => panic!("still pending!!!"),
    }
}

/// Run a service.
pub(crate) fn run<S, T>(mut service: S, req: T) -> Result<S::Response, S::Error>
where
    S: Service<T>,
{
    oneshot(service.call(req))
}
