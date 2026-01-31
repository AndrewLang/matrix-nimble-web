use crate::http::context::HttpContext;
use crate::pipeline::middleware::Middleware;
use crate::pipeline::next::Next;
use crate::pipeline::pipeline::PipelineError;
use async_trait::async_trait;

pub struct ValidationMiddleware;

impl ValidationMiddleware {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Middleware for ValidationMiddleware {
    async fn handle(&self, context: &mut HttpContext, next: Next<'_>) -> Result<(), PipelineError> {
        let _ = context;
        next.run(context).await
    }
}
