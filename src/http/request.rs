use crate::http::headers::HttpHeaders;
use crate::http::request_body::RequestBody;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    method: String,
    path: String,
    headers: HttpHeaders,
    body: RequestBody,
}

impl HttpRequest {
    pub fn new(method: &str, path: &str) -> Self {
        Self {
            method: method.to_string(),
            path: path.to_string(),
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
