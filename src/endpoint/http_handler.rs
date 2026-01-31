use async_trait::async_trait;

use crate::http::context::HttpContext;
use crate::pipeline::pipeline::PipelineError;
use crate::result::into_response::ResponseValue;

#[async_trait]
pub trait HttpHandler: Send + Sync {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError>;
}
