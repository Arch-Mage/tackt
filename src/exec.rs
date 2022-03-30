#![allow(dead_code)]

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;
use std::task::Wake;
use std::thread;
use std::thread::Thread;

pub(crate) struct Waker(Thread);

impl Waker {
    pub(crate) fn new() -> Arc<Waker> {
        Arc::new(Waker(thread::current()))
    }
}

impl Wake for Waker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

pub(crate) struct Executor<'a, F> {
    fut: Pin<Box<F>>,
    ctx: Context<'a>,
}

impl<'a, F> Executor<'a, F>
where
    F: Future,
{
    pub(crate) fn new(future: F, waker: &'a std::task::Waker) -> Executor<'a, F> {
        Executor {
            fut: Box::pin(future),
            ctx: Context::from_waker(waker),
        }
    }

    pub(crate) fn run(&mut self) -> Poll<F::Output> {
        self.fut.as_mut().poll(&mut self.ctx)
    }
}

pub(crate) fn oneshot<T>(future: impl Future<Output = T>) -> T {
    let waker = Waker::new().into();
    let mut exec = Executor::new(future, &waker);
    match exec.run() {
        Poll::Ready(out) => out,
        Poll::Pending => panic!("future still pending"),
    }
}
