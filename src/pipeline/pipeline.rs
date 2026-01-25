use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use crate::http::context::HttpContext;
use crate::pipeline::middleware::{DynMiddleware, Middleware};
use crate::pipeline::next::Next;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineError {
    Message(String),
}

impl PipelineError {
    pub fn message(message: &str) -> Self {
        Self::Message(message.to_string())
    }
}

pub struct Pipeline {
    middleware: Vec<Box<dyn DynMiddleware>>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            middleware: Vec::new(),
        }
    }

    pub fn add<M: Middleware + 'static>(&mut self, middleware: M) {
        self.middleware.push(Box::new(middleware));
    }

    pub fn run(&self, context: &mut HttpContext) -> Result<(), PipelineError> {
        Self::block_on(self.run_async(context))
    }

    pub async fn run_async(&self, context: &mut HttpContext) -> Result<(), PipelineError> {
        let next = Next::new(self, 0);
        next.run(context).await
    }

    pub(crate) fn middleware(&self) -> &[Box<dyn DynMiddleware>] {
        &self.middleware
    }

    fn block_on<F: Future>(mut future: F) -> F::Output {
        let waker = unsafe { Waker::from_raw(Self::raw_waker()) };
        let mut context = Context::from_waker(&waker);
        let mut future = unsafe { Pin::new_unchecked(&mut future) };

        loop {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(value) => return value,
                Poll::Pending => continue,
            }
        }
    }

    fn raw_waker() -> RawWaker {
        fn no_op(_: *const ()) {}
        fn clone(_: *const ()) -> RawWaker {
            Pipeline::raw_waker()
        }
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, no_op, no_op, no_op);
        RawWaker::new(std::ptr::null(), &VTABLE)
    }
}
