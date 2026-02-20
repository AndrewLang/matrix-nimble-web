use async_trait::async_trait;
use nimble_web::controller::route::HttpRoute;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::registry::EndpointRegistry;
use nimble_web::endpoint::route::EndpointRoute;
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

struct TaggedGetWithPolicy;

#[async_trait]
#[get("/attr/get-auth", policy = Policy::Authenticated)]
impl HttpHandler for TaggedGetWithPolicy {
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
fn get_attribute_accepts_policy_expression() {
    let route = TaggedGetWithPolicy::endpoint();
    assert_eq!(
        route.endpoint.metadata().policy(),
        Some(&Policy::Authenticated)
    );
}

#[test]
fn attribute_route_builder_supports_customization() {
    let route = TaggedPost::route()
        .with_policy(Policy::Custom("test".to_string()))
        .build();

    assert_eq!(route.route.method(), "POST");
    assert_eq!(route.route.path(), "/attr/post");
}

#[test]
fn attribute_routes_register_automatically() {
    let mut registry = EndpointRegistry::new();
    registry.register_attribute_routes();

    assert!(registry
        .routes()
        .iter()
        .any(|route| route.method() == "GET" && route.path() == "/attr/get"));
    assert!(registry
        .routes()
        .iter()
        .any(|route| route.method() == "POST" && route.path() == "/attr/post"));
}

#[test]
fn attribute_routes_do_not_duplicate_existing_routes() {
    let mut registry = EndpointRegistry::new();
    registry.add_endpoint_route(EndpointRoute::get("/attr/get", TaggedGet).build());
    registry.register_attribute_routes();

    let get_count = registry
        .routes()
        .iter()
        .filter(|route| route.method() == "GET" && route.path() == "/attr/get")
        .count();

    assert_eq!(get_count, 1);
}
