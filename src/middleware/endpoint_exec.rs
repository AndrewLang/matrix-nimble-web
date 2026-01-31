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

use async_trait::async_trait;

#[async_trait]
impl Middleware for EndpointExecutionMiddleware {
    async fn handle(&self, context: &mut HttpContext, next: Next<'_>) -> Result<(), PipelineError> {
        log::debug!("EndpointExecutionMiddleware: {}", context.request().path());
        if let Some(endpoint) = context.endpoint().cloned() {
            endpoint.invoke(context).await?;
        } else {
            log::debug!(
                "❌ No endpoint found for request {} {}",
                context.request().method(),
                context.request().path()
            );
        }

        next.run(context).await
    }
}
