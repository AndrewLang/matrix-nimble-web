use nimble_web::config::ConfigBuilder;
use nimble_web::di::ServiceContainer;
use nimble_web::http::context::HttpContext;
use nimble_web::http::request::HttpRequest;
use nimble_web::identity::claims::Claims;
use nimble_web::identity::context::IdentityContext;
use nimble_web::identity::kind::IdentityKind;
use nimble_web::identity::method::AuthMethod;
use nimble_web::pipeline::pipeline::Pipeline;
use nimble_web::security::auth::AuthenticationMiddleware;
use nimble_web::testkit::auth::provide_mock_token_service;

fn make_context() -> HttpContext {
    let mut container = ServiceContainer::new();
    provide_mock_token_service(&mut container);
    HttpContext::new(
        HttpRequest::new("GET", "/secure"),
        container.build(),
        ConfigBuilder::new().build(),
    )
}

fn run_middleware(context: &mut HttpContext) {
    let mut pipeline = Pipeline::new();
    pipeline.add(AuthenticationMiddleware::new());
    let _ = pipeline.run(context);
}

#[test]
fn bearer_token_creates_authenticated_identity() {
    let mut context = make_context();
    context
        .request_mut()
        .headers_mut()
        .insert("authorization", "Bearer bearer-id");

    run_middleware(&mut context);

    let identity = context.get::<IdentityContext>().expect("identity inserted");
    assert_eq!(identity.identity().subject(), "bearer-id");
    assert_eq!(identity.identity().kind(), IdentityKind::User);
    assert_eq!(identity.identity().auth_method(), AuthMethod::Bearer);
    assert!(identity.is_authenticated());
}

#[test]
fn missing_header_is_anonymous() {
    let mut context = make_context();
    run_middleware(&mut context);

    let identity = context.get::<IdentityContext>().expect("identity inserted");
    assert_eq!(identity.identity().kind(), IdentityKind::Anonymous);
    assert_eq!(identity.identity().subject(), "anonymous");
    assert_eq!(identity.identity().auth_method(), AuthMethod::Anonymous);
    assert!(!identity.is_authenticated());
}

#[test]
fn malformed_header_still_anonymous() {
    let mut context = make_context();
    context
        .request_mut()
        .headers_mut()
        .insert("authorization", "Basic nope");

    run_middleware(&mut context);

    let identity = context.get::<IdentityContext>().expect("identity inserted");
    assert_eq!(identity.identity().kind(), IdentityKind::Anonymous);
    assert!(!identity.is_authenticated());
}
