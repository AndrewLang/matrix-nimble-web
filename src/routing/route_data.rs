use std::collections::HashMap;

use crate::routing::route::Route;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RouteData {
    route: Route,
    params: HashMap<String, String>,
}

impl RouteData {
    pub fn new(route: Route, params: HashMap<String, String>) -> Self {
        Self { route, params }
    }

    pub fn route(&self) -> &Route {
        &self.route
    }

    pub fn params(&self) -> &HashMap<String, String> {
        &self.params
    }
}
