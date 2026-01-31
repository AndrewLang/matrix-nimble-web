use async_trait::async_trait;
use std::sync::Arc;

use crate::endpoint::http_endpoint_handler::HttpEndpointHandler;
use crate::endpoint::http_handler::HttpHandler;
use crate::endpoint::registry::EndpointRegistry;
use crate::http::context::HttpContext;
use crate::pipeline::middleware::Middleware;
use crate::pipeline::next::Next;
use crate::pipeline::pipeline::PipelineError;
use crate::result::into_response::ResponseValue;
use crate::result::IntoResponse;

pub struct ControllerInvoker<C> {
    controller: Arc<C>,
}

impl<C> ControllerInvoker<C>
where
    C: Send + Sync + 'static,
{
    pub fn new(controller: Arc<C>) -> Self {
        Self { controller }
    }

    pub fn handler<F, R>(&self, func: F) -> HttpEndpointHandler
    where
        F: Fn(&C, &mut HttpContext) -> R + Send + Sync + 'static,
        R: IntoResponse + Send + Sync + 'static,
    {
        HttpEndpointHandler::new(ControllerHandler {
            controller: self.controller.clone(),
            func: Arc::new(func),
        })
    }
}

struct ControllerHandler<C, F> {
    controller: Arc<C>,
    func: Arc<F>,
}

#[async_trait]
impl<C, F, R> HttpHandler for ControllerHandler<C, F>
where
    C: Send + Sync + 'static,
    F: Fn(&C, &mut HttpContext) -> R + Send + Sync + 'static,
    R: IntoResponse + Send + Sync + 'static,
{
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let value = (self.func)(self.controller.as_ref(), context);
        Ok(ResponseValue::new(value))
    }
}

pub struct ControllerInvokerMiddleware {
    registry: Arc<EndpointRegistry>,
}

impl ControllerInvokerMiddleware {
    pub fn new(registry: Arc<EndpointRegistry>) -> Self {
        Self { registry }
    }
}

#[async_trait]
impl Middleware for ControllerInvokerMiddleware {
    async fn handle(&self, context: &mut HttpContext, next: Next<'_>) -> Result<(), PipelineError> {
        if let Some(route_data) = context.route() {
            if let Some(endpoint) = self.registry.find_endpoint(route_data.route()) {
                context.set_endpoint(endpoint);
            }
        }

        next.run(context).await
    }
}
