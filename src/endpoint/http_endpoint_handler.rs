use std::sync::Arc;

use crate::endpoint::http_handler::HttpHandler;
use crate::http::context::HttpContext;
use crate::pipeline::pipeline::PipelineError;
use crate::result::into_response::ResponseValue;

#[derive(Clone)]
pub struct HttpEndpointHandler {
    inner: Arc<dyn HttpHandler>,
}

impl HttpEndpointHandler {
    pub fn new<H>(handler: H) -> Self
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        Self {
            inner: Arc::new(handler),
        }
    }

    pub(crate) async fn invoke(
        &self,
        ctx: &mut HttpContext,
    ) -> Result<ResponseValue, PipelineError> {
        self.inner.invoke(ctx).await
    }
}
