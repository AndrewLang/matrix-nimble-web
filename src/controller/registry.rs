use std::sync::Arc;

use crate::controller::controller::Controller;
use crate::endpoint::endpoint::Endpoint;
use crate::endpoint::http_endpoint::HttpEndpoint;
use crate::endpoint::http_endpoint_handler::HttpEndpointHandler;
use crate::endpoint::http_handler::HttpHandler;
use crate::endpoint::metadata::EndpointMetadata;

use crate::routing::route::Route;
use crate::security::policy::Policy;

use crate::endpoint::route::EndpointRoute;

pub struct ControllerRegistry {
    routes: Vec<Route>,
    endpoints: Vec<Arc<dyn Endpoint>>,
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
        let endpoint = Arc::new(HttpEndpoint::new(
            HttpEndpointHandler::new(handler),
            metadata,
        ));
        self.add_route(route, endpoint);
    }

    pub fn add_with_policy<H>(&mut self, method: &str, path: &str, handler: H, policy: Policy)
    where
        H: HttpHandler + Send + Sync + 'static,
    {
        let route = Route::new(method, path);
        let metadata = EndpointMetadata::new(method, path).require_policy(policy);
        let endpoint = Arc::new(HttpEndpoint::new(
            HttpEndpointHandler::new(handler),
            metadata,
        ));
        self.add_route(route, endpoint);
    }

    pub fn add_route(&mut self, route: Route, endpoint: Arc<dyn Endpoint>) {
        self.routes.push(route);
        self.endpoints.push(endpoint);
    }

    pub fn add_endpoint_route(&mut self, endpoint_route: EndpointRoute) {
        self.add_route(endpoint_route.route, endpoint_route.endpoint);
    }

    pub fn routes(&self) -> &[Route] {
        &self.routes
    }

    pub fn endpoints(&self) -> &[Arc<dyn Endpoint>] {
        &self.endpoints
    }

    pub fn find_endpoint(&self, route: &Route) -> Option<Arc<dyn Endpoint>> {
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
