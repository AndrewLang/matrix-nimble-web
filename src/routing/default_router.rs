use crate::http::request::HttpRequest;
use crate::routing::route::Route;
use crate::routing::route_data::RouteData;
use crate::routing::router::Router;

pub struct DefaultRouter {
    routes: Vec<Route>,
}

impl DefaultRouter {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    pub fn routes(&self) -> &Vec<Route> {
        &self.routes
    }

    pub fn log_routes(&self) {
        for route in &self.routes {
            log::info!("Route: {} {}", route.method(), route.path());
        }
    }
}

impl Router for DefaultRouter {
    fn add_route(&mut self, route: Route) {
        self.routes.push(route);
    }

    fn match_request(&self, request: &HttpRequest) -> Option<RouteData> {
        for route in &self.routes {
            if let Some(data) = route.match_path(request.method(), request.path()) {
                return Some(data);
            }
        }

        None
    }
}
