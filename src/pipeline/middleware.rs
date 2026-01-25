use std::future::Future;
use std::pin::Pin;

use crate::http::context::HttpContext;
use crate::pipeline::next::Next;
use crate::pipeline::pipeline::PipelineError;

pub(crate) type MiddlewareFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), PipelineError>> + 'a>>;

#[allow(async_fn_in_trait)]
pub trait Middleware: Send + Sync {
    async fn handle(&self, context: &mut HttpContext, next: Next<'_>) -> Result<(), PipelineError>;
}

pub(crate) trait DynMiddleware: Send + Sync {
    fn handle<'a>(&'a self, context: &'a mut HttpContext, next: Next<'a>) -> MiddlewareFuture<'a>;
}

impl<T> DynMiddleware for T
where
    T: Middleware + Send + Sync,
{
    fn handle<'a>(&'a self, context: &'a mut HttpContext, next: Next<'a>) -> MiddlewareFuture<'a> {
        Box::pin(async move { T::handle(self, context, next).await })
    }
}
