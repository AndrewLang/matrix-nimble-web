use crate::http::headers::HttpHeaders;
use crate::http::request_body::RequestBody;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    method: String,
    path: String,
    query: Option<String>,
    headers: HttpHeaders,
    body: RequestBody,
}

impl HttpRequest {
    pub fn new(method: &str, path: &str) -> Self {
        Self {
            method: method.to_string(),
            path: path.to_string(),
            query: None,
            headers: HttpHeaders::new(),
            body: RequestBody::Empty,
        }
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn set_query(&mut self, query: Option<String>) {
        self.query = query;
    }

    pub fn query(&self) -> Option<&str> {
        self.query.as_deref()
    }

    pub fn query_param(&self, key: &str) -> Option<String> {
        self.query_params().get(key).cloned()
    }

    pub fn query_params(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        let Some(query) = &self.query else {
            return map;
        };

        for pair in query.split('&') {
            if pair.is_empty() {
                continue;
            }

            let mut parts = pair.splitn(2, '=');
            let k = parts.next().unwrap_or("").trim();
            if k.is_empty() {
                continue;
            }
            let v = parts.next().unwrap_or("").trim();
            map.insert(k.to_string(), v.to_string());
        }

        map
    }

    pub fn headers(&self) -> &HttpHeaders {
        &self.headers
    }

    pub fn headers_mut(&mut self) -> &mut HttpHeaders {
        &mut self.headers
    }

    pub fn set_body(&mut self, body: RequestBody) {
        self.body = body;
    }

    pub fn body(&self) -> &RequestBody {
        &self.body
    }
}
