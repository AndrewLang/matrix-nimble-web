use async_trait::async_trait;
use nimble_web::controller::route::HttpRoute;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::http::context::HttpContext;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::into_response::ResponseValue;
use nimble_web::security::policy::Policy;
use nimble_web::{get, post};

struct TaggedGet;

#[async_trait]
#[get("/attr/get")]
impl HttpHandler for TaggedGet {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        Err(PipelineError::message("not used"))
    }
}

struct TaggedPost;

#[async_trait]
#[post(path = "/attr/post")]
impl HttpHandler for TaggedPost {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        Err(PipelineError::message("not used"))
    }
}

#[test]
fn get_attribute_generates_route_metadata() {
    let route = TaggedGet::endpoint();
    assert_eq!(route.route.method(), "GET");
    assert_eq!(route.route.path(), "/attr/get");
}

#[test]
fn attribute_route_builder_supports_customization() {
    let route = TaggedPost::route()
        .with_policy(Policy::Custom("test".to_string()))
        .build();

    assert_eq!(route.route.method(), "POST");
    assert_eq!(route.route.path(), "/attr/post");
}
