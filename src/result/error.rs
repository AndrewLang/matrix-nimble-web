use crate::http::context::HttpContext;
use crate::http::response_body::ResponseBody;
use crate::result::into_response::IntoResponse;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpError {
    status: u16,
    message: String,
}

impl HttpError {
    pub fn new(status: u16, message: &str) -> Self {
        Self {
            status,
            message: message.to_string(),
        }
    }

    pub fn status(&self) -> u16 {
        self.status
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for HttpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "HTTP {}: {}", self.status, self.message)
    }
}

impl Error for HttpError {}

impl IntoResponse for HttpError {
    fn into_response(self, context: &mut HttpContext) {
        let response = context.response_mut();
        response.set_status(self.status);
        response.set_body(ResponseBody::Text(self.message));
    }
}

impl<T> IntoResponse for Result<T, HttpError>
where
    T: IntoResponse,
{
    fn into_response(self, context: &mut HttpContext) {
        match self {
            Ok(value) => value.into_response(context),
            Err(error) => error.into_response(context),
        }
    }
}
