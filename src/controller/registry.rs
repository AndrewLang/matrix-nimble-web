use crate::controller::controller::Controller;
use crate::endpoint::endpoint::Endpoint;
use crate::endpoint::http_handler::HttpHandler;
use crate::endpoint::kind::{EndpointKind, HttpEndpointHandler};
use crate::endpoint::metadata::EndpointMetadata;
use crate::routing::route::Route;
use crate::security::policy::Policy;

#[derive(Default)]
pub struct ControllerRegistry {
    routes: Vec<Route>,
    endpoints: Vec<Endpoint>,
}

pub struct ControllerActionBuilder<'a> {
    registry: &'a mut ControllerRegistry,
    method: &'static str,
    path: String,
    endpoint: Endpoint,
}

impl<'a> ControllerActionBuilder<'a> {
    pub fn validate<T>(mut self, validator: T) -> Self
    where
        T: crate::validation::AnyValidator + 'static,
    {
        let metadata = self.endpoint.metadata().clone().add_validator(validator);
        self.endpoint = Endpoint::new(self.endpoint.kind().clone(), metadata);
        self
    }

    pub fn register(self) {
        let route = Route::new(self.method, &self.path);
        self.registry.add_route(route, self.endpoint);
    }
}

impl ControllerRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<C: Controller>(&mut self) {
        C::register(self);
    }

    pub fn add<H>(&mut self, method: &str, path: &str, handler: H)
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        let route = Route::new(method, path);
        let metadata = EndpointMetadata::new(method, path);
        let endpoint = Endpoint::new(
            EndpointKind::Http(HttpEndpointHandler::new(handler)),
            metadata,
        );
        self.add_route(route, endpoint);
    }

    pub fn post<H>(&mut self, path: &str, handler: H) -> ControllerActionBuilder<'_>
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        self.action("POST", path, handler)
    }

    pub fn get<H>(&mut self, path: &str, handler: H) -> ControllerActionBuilder<'_>
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        self.action("GET", path, handler)
    }

    fn action<H>(
        &mut self,
        method: &'static str,
        path: &str,
        handler: H,
    ) -> ControllerActionBuilder<'_>
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        let metadata = EndpointMetadata::new(method, path);
        let endpoint = Endpoint::new(
            EndpointKind::Http(HttpEndpointHandler::new(handler)),
            metadata,
        );
        ControllerActionBuilder {
            registry: self,
            method,
            path: path.to_string(),
            endpoint,
        }
    }

    pub fn add_with_policy<H>(&mut self, method: &str, path: &str, handler: H, policy: Policy)
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        let route = Route::new(method, path);
        let metadata = EndpointMetadata::new(method, path).require_policy(policy);
        let endpoint = Endpoint::new(
            EndpointKind::Http(HttpEndpointHandler::new(handler)),
            metadata,
        );
        self.add_route(route, endpoint);
    }

    pub fn add_route(&mut self, route: Route, endpoint: Endpoint) {
        self.routes.push(route);
        self.endpoints.push(endpoint);
    }

    pub fn routes(&self) -> &[Route] {
        &self.routes
    }

    pub fn endpoints(&self) -> &[Endpoint] {
        &self.endpoints
    }

    pub fn find_endpoint(&self, route: &Route) -> Option<Endpoint> {
        self.routes
            .iter()
            .position(|candidate| candidate == route)
            .and_then(|index| self.endpoints.get(index).cloned())
    }
}
