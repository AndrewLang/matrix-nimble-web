use crate::http::context::HttpContext;
use crate::http::response_body::ResponseBody;
use crate::result::into_response::IntoResponse;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationError {
    message: String,
}

impl ValidationError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl IntoResponse for ValidationError {
    fn into_response(self, context: &mut HttpContext) {
        let response = context.response_mut();
        response.set_status(400);
        response.set_body(ResponseBody::Text(self.message));
        response
            .headers_mut()
            .insert("content-type", "text/plain; charset=utf-8");
    }
}
