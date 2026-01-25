use crate::http::headers::HttpHeaders;
use crate::http::response_body::ResponseBody;

#[derive(Debug, Clone)]
pub struct HttpResponse {
    status: u16,
    headers: HttpHeaders,
    body: ResponseBody,
}

impl Default for HttpResponse {
    fn default() -> Self {
        Self {
            status: 200,
            headers: HttpHeaders::new(),
            body: ResponseBody::Empty,
        }
    }
}

impl HttpResponse {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn status(&self) -> u16 {
        self.status
    }

    pub fn set_status(&mut self, status: u16) {
        self.status = status;
    }

    pub fn headers(&self) -> &HttpHeaders {
        &self.headers
    }

    pub fn headers_mut(&mut self) -> &mut HttpHeaders {
        &mut self.headers
    }

    pub fn set_body(&mut self, body: ResponseBody) {
        self.body = body;
    }

    pub fn body(&self) -> &ResponseBody {
        &self.body
    }
}
