use crate::http::request::HttpRequest;
use crate::routing::route::Route;
use crate::routing::route_data::RouteData;

pub trait Router {
    fn add_route(&mut self, route: Route);

    fn match_request(&self, request: &HttpRequest) -> Option<RouteData>;
}
