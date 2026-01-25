use crate::endpoint::kind::EndpointKind;
use crate::http::context::HttpContext;
use crate::pipeline::middleware::Middleware;
use crate::pipeline::next::Next;
use crate::pipeline::pipeline::PipelineError;

pub struct EndpointExecutionMiddleware;

impl EndpointExecutionMiddleware {
    pub fn new() -> Self {
        Self
    }
}

#[allow(async_fn_in_trait)]
impl Middleware for EndpointExecutionMiddleware {
    async fn handle(&self, context: &mut HttpContext, next: Next<'_>) -> Result<(), PipelineError> {
        if let Some(endpoint) = context.endpoint().cloned() {
            match endpoint.kind() {
                EndpointKind::Http(handler) => {
                    handler.invoke(context).await?;
                }
                EndpointKind::WebSocket(_handler) => {}
            }
        }

        next.run(context).await
    }
}
