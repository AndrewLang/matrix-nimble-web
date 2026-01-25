use std::collections::HashMap;
use std::fmt;

use crate::routing::route_data::RouteData;

#[derive(Clone, PartialEq, Eq)]
pub struct Route {
    method: String,
    path: String,
}

impl Route {
    pub fn new(method: &str, path: &str) -> Self {
        Self {
            method: method.to_string(),
            path: path.to_string(),
        }
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn match_path(&self, method: &str, path: &str) -> Option<RouteData> {
        if self.method != method {
            return None;
        }

        let pattern_segments = self.split_segments(&self.path);
        let path_segments = self.split_segments(path);

        if pattern_segments.len() != path_segments.len() {
            return None;
        }

        let mut params = HashMap::new();

        for (pattern_segment, path_segment) in pattern_segments
            .iter()
            .copied()
            .zip(path_segments.iter().copied())
        {
            if let Some(param_name) = self.param_name(pattern_segment) {
                params.insert(param_name.to_string(), path_segment.to_string());
                continue;
            }

            if pattern_segment != path_segment {
                return None;
            }
        }

        Some(RouteData::new(self.clone(), params))
    }

    fn split_segments<'a>(&self, path: &'a str) -> Vec<&'a str> {
        let trimmed = path.trim_matches('/');
        if trimmed.is_empty() {
            Vec::new()
        } else {
            trimmed.split('/').collect()
        }
    }

    fn param_name<'a>(&self, segment: &'a str) -> Option<&'a str> {
        if segment.len() >= 3 && segment.starts_with('{') && segment.ends_with('}') {
            Some(&segment[1..segment.len() - 1])
        } else {
            None
        }
    }
}

impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Route")
            .field("method", &self.method)
            .field("path", &self.path)
            .finish()
    }
}
