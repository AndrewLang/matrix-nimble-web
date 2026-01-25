use std::future::Future;
use std::pin::Pin;

use crate::http::context::HttpContext;
use crate::pipeline::pipeline::PipelineError;
use crate::result::into_response::ResponseValue;

pub(crate) type HttpHandlerFuture<'a> =
    Pin<Box<dyn Future<Output = Result<ResponseValue, PipelineError>> + 'a>>;

#[allow(async_fn_in_trait)]
pub trait HttpHandler: Send + Sync {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError>;
}

pub(crate) trait DynHttpHandler: Send + Sync {
    fn invoke<'a>(&'a self, context: &'a mut HttpContext) -> HttpHandlerFuture<'a>;
}

impl<T> DynHttpHandler for T
where
    T: HttpHandler + Send + Sync,
{
    fn invoke<'a>(&'a self, context: &'a mut HttpContext) -> HttpHandlerFuture<'a> {
        Box::pin(async move { T::invoke(self, context).await })
    }
}
