use crate::http::context::HttpContext;
use crate::http::response_body::ResponseBody;
use crate::result::into_response::IntoResponse;

fn write_text_response(text: String, ctx: &mut HttpContext) {
    let response = ctx.response_mut();
    response.set_status(200);
    response.set_body(ResponseBody::Text(text));
    response
        .headers_mut()
        .insert("content-type", "text/plain; charset=utf-8");
}

impl IntoResponse for String {
    fn into_response(self, ctx: &mut HttpContext) {
        write_text_response(self, ctx);
    }
}

impl IntoResponse for &'static str {
    fn into_response(self, ctx: &mut HttpContext) {
        write_text_response(self.to_string(), ctx);
    }
}
