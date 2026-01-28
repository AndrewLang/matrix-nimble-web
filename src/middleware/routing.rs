use std::sync::Arc;

use crate::endpoint::registry::EndpointRegistry;
use crate::http::context::HttpContext;
use crate::pipeline::middleware::Middleware;
use crate::pipeline::next::Next;
use crate::pipeline::pipeline::PipelineError;
use crate::routing::router::Router;

pub struct RoutingMiddleware {
    router: Arc<dyn Router + Send + Sync>,
    endpoint_registry: Option<Arc<EndpointRegistry>>,
}

impl RoutingMiddleware {
    pub fn new<R>(router: R) -> Self
    where
        R: Router + Send + Sync + 'static,
    {
        Self {
            router: Arc::new(router),
            endpoint_registry: None,
        }
    }

    pub fn with_registry<R>(router: R, registry: Arc<EndpointRegistry>) -> Self
    where
        R: Router + Send + Sync + 'static,
    {
        Self {
            router: Arc::new(router),
            endpoint_registry: Some(registry),
        }
    }
}

impl Middleware for RoutingMiddleware {
    async fn handle(&self, context: &mut HttpContext, next: Next<'_>) -> Result<(), PipelineError> {
        if let Some(route_data) = self.router.match_request(context.request()) {
            log::debug!("Route matched: {}", route_data.route());
            if let Some(registry) = self.endpoint_registry.as_ref() {
                if let Some(endpoint) = registry.find_endpoint(route_data.route()) {
                    context.set_endpoint(endpoint);
                }
            }
            context.set_route(route_data);
        }

        next.run(context).await
    }
}
