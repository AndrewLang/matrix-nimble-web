use std::future::Future;
use std::pin::Pin;

use crate::endpoint::metadata::EndpointMetadata;
use crate::http::context::HttpContext;
use crate::pipeline::pipeline::PipelineError;

pub type EndpointFuture<'a> = Pin<Box<dyn Future<Output = Result<(), PipelineError>> + Send + 'a>>;

pub trait Endpoint: Send + Sync {
    fn metadata(&self) -> &EndpointMetadata;
    fn invoke<'a>(&'a self, context: &'a mut HttpContext) -> EndpointFuture<'a>;
}
