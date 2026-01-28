use crate::controller::controller::Controller;
use crate::endpoint::endpoint::Endpoint;
use crate::endpoint::http_handler::HttpHandler;
use crate::endpoint::kind::{EndpointKind, HttpEndpointHandler};
use crate::endpoint::metadata::EndpointMetadata;

use crate::routing::route::Route;
use crate::security::policy::Policy;

pub struct EndpointRoute {
    pub route: Route,
    pub endpoint: Endpoint,
}

impl EndpointRoute {
    pub fn new(route: Route, endpoint: Endpoint) -> Self {
        Self { route, endpoint }
    }

    pub fn get<H>(path: &str, handler: H) -> RouteBuilder
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        RouteBuilder::new("GET", path, handler)
    }

    pub fn post<H>(path: &str, handler: H) -> RouteBuilder
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        RouteBuilder::new("POST", path, handler)
    }
}

pub struct ControllerRegistry {
    routes: Vec<Route>,
    endpoints: Vec<Endpoint>,
}

pub struct RouteBuilder {
    method: &'static str,
    path: String,
    endpoint: Endpoint,
}

impl RouteBuilder {
    pub fn new<H>(method: &'static str, path: &str, handler: H) -> Self
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        let metadata = EndpointMetadata::new(method, path);
        let endpoint = Endpoint::new(
            EndpointKind::Http(HttpEndpointHandler::new(handler)),
            metadata,
        );
        Self {
            method,
            path: path.to_string(),
            endpoint,
        }
    }

    pub fn validate<T>(mut self, validator: T) -> Self
    where
        T: crate::validation::AnyValidator + 'static,
    {
        let metadata = self.endpoint.metadata().clone().add_validator(validator);
        self.endpoint = Endpoint::new(self.endpoint.kind().clone(), metadata);
        self
    }

    pub fn with_policy(mut self, policy: Policy) -> Self {
        let metadata = self.endpoint.metadata().clone().require_policy(policy);
        self.endpoint = Endpoint::new(self.endpoint.kind().clone(), metadata);
        self
    }

    pub fn build(self) -> EndpointRoute {
        let route = Route::new(self.method, &self.path);
        EndpointRoute::new(route, self.endpoint)
    }
}

impl ControllerRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<C: Controller>(&mut self) {
        let routes = C::routes();
        for endpoint_route in routes {
            self.add_endpoint_route(endpoint_route);
        }
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

    pub fn add_endpoint_route(&mut self, endpoint_route: EndpointRoute) {
        self.add_route(endpoint_route.route, endpoint_route.endpoint);
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

impl Default for ControllerRegistry {
    fn default() -> Self {
        Self {
            routes: Vec::new(),
            endpoints: Vec::new(),
        }
    }
}
