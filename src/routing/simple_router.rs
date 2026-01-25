use crate::http::request::HttpRequest;
use crate::routing::route::Route;
use crate::routing::route_data::RouteData;
use crate::routing::router::Router;

pub struct SimpleRouter {
    routes: Vec<Route>,
}

impl SimpleRouter {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }
}

impl Router for SimpleRouter {
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
