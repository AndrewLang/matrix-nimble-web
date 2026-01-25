use crate::app::application::Application;
use crate::controller::controller::Controller;
use crate::controller::registry::ControllerRegistry;
use crate::pipeline::middleware::Middleware;
use crate::pipeline::pipeline::Pipeline;
use crate::routing::router::Router;
use crate::routing::simple_router::SimpleRouter;
use crate::security::auth::AuthenticationMiddleware;
use crate::security::policy::AuthorizationMiddleware;
use crate::validation::ValidationMiddleware;

pub struct AppBuilder {
    pipeline: Pipeline,
    controller_registry: ControllerRegistry,
    router: SimpleRouter,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            pipeline: Pipeline::new(),
            controller_registry: ControllerRegistry::new(),
            router: SimpleRouter::new(),
        }
    }

    pub fn use_middleware<M: Middleware + 'static>(&mut self, middleware: M) -> &mut Self {
        self.pipeline.add(middleware);
        self
    }

    pub fn add_controller<T: Controller>(&mut self) -> &mut Self {
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

    pub fn use_authentication(&mut self) -> &mut Self {
        self.pipeline.add(AuthenticationMiddleware::new());
        self
    }

    pub fn use_authorization(&mut self) -> &mut Self {
        self.pipeline.add(AuthorizationMiddleware::new());
        self
    }

    pub fn use_validation(&mut self) -> &mut Self {
        self.pipeline.add(ValidationMiddleware::new());
        self
    }

    pub fn build(self) -> Application {
        Application::new(self.pipeline)
    }
}
