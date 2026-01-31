use std::collections::HashMap;

use crate::http::request::HttpRequest;
use crate::routing::route::Route;
use crate::routing::route_data::RouteData;
use crate::routing::router::Router;

#[derive(Clone)]
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
        fn normalize_path(path: &str) -> String {
            let path = path.split('?').next().unwrap_or(path).trim_end_matches('/');
            let trimmed = path.trim_start_matches('/');
            if trimmed.is_empty() {
                return "/".to_string();
            }

            let parts: Vec<&str> = trimmed.split('/').collect();
            let mapped: Vec<String> = parts
                .into_iter()
                .map(|s| {
                    if s.parse::<i64>().is_ok() {
                        "{id}".to_string()
                    } else if s.len() == 36 && s.matches('-').count() == 4 {
                        "{id}".to_string()
                    } else {
                        s.to_string()
                    }
                })
                .collect();

            if mapped.len() >= 2 {
                format!("{}/{}", mapped[0], mapped[1])
            } else {
                mapped.join("/")
            }
        }

        log::info!("");
        log::info!("Registered routes (grouped):");
        let mut groups: HashMap<String, Vec<&Route>> = HashMap::new();
        for route in &self.routes {
            let key = normalize_path(route.path());
            groups.entry(key).or_default().push(route);
        }

        for (key, routes) in groups {
            let count = routes.len();
            let mut methods: Vec<String> = routes.iter().map(|r| r.method().to_string()).collect();
            methods.sort();
            methods.dedup();

            log::info!(
                "⇒ {} — {} route(s) — methods: {}",
                key,
                count,
                methods.join(", ")
            );

            for r in routes.iter() {
                log::info!("    ⇢  Example: {:<8} {}", r.method(), r.path());
            }

            log::info!("");
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
