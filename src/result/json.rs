use crate::http::context::HttpContext;
use crate::http::response_body::ResponseBody;
use crate::result::into_response::IntoResponse;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq)]
pub struct Json<T>(pub T);

impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    fn into_response(self, context: &mut HttpContext) {
        let response = context.response_mut();
        match serde_json::to_string(&self.0) {
            Ok(payload) => {
                response.set_status(200);
                response.set_body(ResponseBody::Text(payload));
                response
                    .headers_mut()
                    .insert("content-type", "application/json");
            }
            Err(_) => {
                response.set_status(500);
                response.set_body(ResponseBody::Empty);
            }
        }
    }
}
