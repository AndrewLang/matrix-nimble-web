use crate::http::headers::HttpHeaders;
use crate::http::request::HttpRequest;
use crate::http::request_body::RequestBody;

pub struct HttpRequestBuilder {
    method: String,
    path: String,
    headers: HttpHeaders,
    body: Option<RequestBody>,
}

impl HttpRequestBuilder {
    pub fn new(method: &str, path: &str) -> Self {
        Self {
            method: method.to_string(),
            path: path.to_string(),
            headers: HttpHeaders::new(),
            body: None,
        }
    }

    pub fn get(path: &str) -> Self {
        Self::new("GET", path)
    }

    pub fn post(path: &str) -> Self {
        Self::new("POST", path)
    }

    pub fn put(path: &str) -> Self {
        Self::new("PUT", path)
    }

    pub fn delete(path: &str) -> Self {
        Self::new("DELETE", path)
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key, value);
        self
    }

    pub fn body(mut self, body: &str) -> Self {
        self.body = Some(RequestBody::Text(body.to_string()));
        self
    }

    pub fn build(self) -> HttpRequest {
        let mut request = HttpRequest::new(&self.method, &self.path);
        for (key, value) in self.headers.iter() {
            request.headers_mut().insert(key, value);
        }
        if let Some(body) = self.body {
            request.set_body(body);
        }
        request
    }
}
