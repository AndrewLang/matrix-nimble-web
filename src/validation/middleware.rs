use crate::http::context::HttpContext;
use crate::pipeline::middleware::Middleware;
use crate::pipeline::next::Next;
use crate::pipeline::pipeline::PipelineError;
use crate::result::into_response::IntoResponse;

pub struct ValidationMiddleware;

impl ValidationMiddleware {
    pub fn new() -> Self {
        Self
    }
}

impl Middleware for ValidationMiddleware {
    async fn handle(&self, context: &mut HttpContext, next: Next<'_>) -> Result<(), PipelineError> {
        let endpoint = match context.endpoint() {
            Some(endpoint) => endpoint,
            None => return next.run(context).await,
        };

        for validator in endpoint.metadata().validators() {
            if let Err(error) = validator.validate(context) {
                error.into_response(context);
                return Ok(());
            }
        }

        next.run(context).await
    }
}
