use crate::controller::controller::Controller;
use crate::endpoint::endpoint::Endpoint;
use crate::endpoint::http_handler::HttpHandler;
use crate::endpoint::kind::{EndpointKind, HttpEndpointHandler};
use crate::endpoint::metadata::EndpointMetadata;
use crate::routing::route::Route;

#[derive(Default)]
pub struct ControllerRegistry {
    routes: Vec<Route>,
    endpoints: Vec<Endpoint>,
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
