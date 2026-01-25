use std::sync::Arc;

use crate::app::builder::AppBuilder;
use crate::controller::controller::Controller;
use crate::controller::registry::ControllerRegistry;
use crate::http::request::HttpRequest;
use crate::http::response::HttpResponse;
use crate::middleware::endpoint_exec::EndpointExecutionMiddleware;
use crate::middleware::routing::RoutingMiddleware;
use crate::pipeline::middleware::Middleware;
use crate::routing::router::Router;
use crate::routing::simple_router::SimpleRouter;
use crate::validation::ValidationMiddleware;

pub struct TestApp {
    builder: Option<AppBuilder>,
    controller_registry: ControllerRegistry,
    router: SimpleRouter,
}

impl TestApp {
    pub fn new() -> Self {
        Self {
            builder: Some(AppBuilder::new()),
            controller_registry: ControllerRegistry::new(),
            router: SimpleRouter::new(),
        }
    }

    pub fn use_middleware<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        if let Some(builder) = self.builder.as_mut() {
            builder.use_middleware(middleware);
        }
        self
    }

    pub fn add_controller<T: Controller>(mut self) -> Self {
        let mut registry = ControllerRegistry::new();
        T::register(&mut registry);
        for (route, endpoint) in registry
            .routes()
            .iter()
            .cloned()
            .zip(registry.endpoints().iter().cloned())
        {
            self.router.add_route(route.clone());
            self.controller_registry.add_route(route, endpoint);
        }
        self
    }

    pub fn use_auth(mut self) -> Self {
        if let Some(builder) = self.builder.as_mut() {
            builder.use_authentication();
            builder.use_authorization();
        }
        self
    }

    pub fn run(self, request: HttpRequest) -> HttpResponse {
        let mut builder = self.builder.expect("test app builder");
        if !self.controller_registry.routes().is_empty() {
            builder.use_middleware(RoutingMiddleware::with_registry(
                self.router,
                Arc::new(self.controller_registry),
            ));
            builder.use_middleware(ValidationMiddleware::new());
            builder.use_middleware(EndpointExecutionMiddleware::new());
        }
        let app = builder.build();
        app.handle_http(request)
    }

    pub async fn run_async(self, request: HttpRequest) -> HttpResponse {
        self.run(request)
    }
}
