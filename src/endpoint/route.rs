use std::sync::Arc;

use crate::endpoint::endpoint::Endpoint;
use crate::endpoint::http_endpoint::HttpEndpoint;
use crate::endpoint::http_endpoint_handler::HttpEndpointHandler;
use crate::endpoint::http_handler::HttpHandler;
use crate::endpoint::metadata::EndpointMetadata;
use crate::routing::route::Route;
use crate::security::policy::Policy;

pub struct EndpointRoute {
    pub route: Route,
    pub endpoint: Arc<dyn Endpoint>,
}

impl EndpointRoute {
    pub fn new(route: Route, endpoint: Arc<dyn Endpoint>) -> Self {
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

    pub fn put<H>(path: &str, handler: H) -> RouteBuilder
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        RouteBuilder::new("PUT", path, handler)
    }

    pub fn delete<H>(path: &str, handler: H) -> RouteBuilder
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        RouteBuilder::new("DELETE", path, handler)
    }
}

pub struct EndpointRouteRegistry {
    pub routes: Vec<EndpointRoute>,
}

impl EndpointRouteRegistry {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    pub fn get<H>(&mut self, path: &str, handler: H)
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        self.add("GET", path, handler);
    }

    pub fn post<H>(&mut self, path: &str, handler: H)
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        self.add("POST", path, handler);
    }

    pub fn put<H>(&mut self, path: &str, handler: H)
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        self.add("PUT", path, handler);
    }

    pub fn delete<H>(&mut self, path: &str, handler: H)
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        self.add("DELETE", path, handler);
    }

    fn add<H>(&mut self, method: &'static str, path: &str, handler: H)
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        let route = RouteBuilder::new(method, path, handler).build();

        self.routes.push(route);
    }
}

pub struct RouteBuilder {
    method: &'static str,
    path: String,
    handler: HttpEndpointHandler,
    metadata: EndpointMetadata,
}

impl RouteBuilder {
    pub fn new<H>(method: &'static str, path: &str, handler: H) -> Self
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        let metadata = EndpointMetadata::new(method, path);
        Self {
            method,
            path: path.to_string(),
            handler: HttpEndpointHandler::new(handler),
            metadata,
        }
    }

    pub fn validate<T>(mut self, validator: T) -> Self
    where
        T: crate::validation::AnyValidator + 'static,
    {
        self.metadata = self.metadata.add_validator(validator);
        self
    }

    pub fn with_policy(mut self, policy: Policy) -> Self {
        self.metadata = self.metadata.require_policy(policy);
        self
    }

    pub fn build(self) -> EndpointRoute {
        let route = Route::new(self.method, &self.path);
        let endpoint = Arc::new(HttpEndpoint::new(self.handler, self.metadata));
        EndpointRoute::new(route, endpoint)
    }
}
