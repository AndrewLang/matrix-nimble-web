use crate::endpoint::http_handler::HttpHandler;
use crate::endpoint::route::{EndpointRoute, RouteBuilder};

pub trait HttpRoute: HttpHandler + Sized + Send + Sync + 'static {
    fn route() -> RouteBuilder;

    fn endpoint() -> EndpointRoute {
        Self::route().build()
    }
}
