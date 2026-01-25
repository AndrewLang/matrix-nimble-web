use std::sync::Arc;

use crate::http::context::HttpContext;
use crate::pipeline::middleware::Middleware;
use crate::pipeline::next::Next;
use crate::pipeline::pipeline::PipelineError;
use crate::routing::router::Router;

pub struct RoutingMiddleware {
    router: Arc<dyn Router + Send + Sync>,
}

impl RoutingMiddleware {
    pub fn new<R>(router: R) -> Self
    where
        R: Router + Send + Sync + 'static,
    {
        Self {
            router: Arc::new(router),
        }
    }
}

impl Middleware for RoutingMiddleware {
    async fn handle(&self, context: &mut HttpContext, next: Next<'_>) -> Result<(), PipelineError> {
        if let Some(route_data) = self.router.match_request(context.request()) {
            context.set_route(route_data);
        }

        next.run(context).await
    }
}
