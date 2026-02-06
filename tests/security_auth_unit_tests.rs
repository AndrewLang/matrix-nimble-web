use std::sync::Arc;

use anyhow::anyhow;
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
use nimble_web::security::token::TokenService;
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

struct RejectingTokenService;

impl TokenService for RejectingTokenService {
    fn create_access_token(
        &self,
        _user: &nimble_web::identity::user::UserIdentity,
    ) -> std::result::Result<String, anyhow::Error> {
        Ok("token".to_string())
    }

    fn validate_access_token(&self, _token: &str) -> std::result::Result<Claims, anyhow::Error> {
        Err(anyhow!("ExpiredSignature"))
    }

    fn create_refresh_token(&self, user_id: &str) -> std::result::Result<String, anyhow::Error> {
        Ok(user_id.to_string())
    }

    fn validate_refresh_token(&self, token: &str) -> std::result::Result<String, anyhow::Error> {
        Ok(token.to_string())
    }

    fn revoke_refresh_token(&self, _token: &str) -> std::result::Result<(), anyhow::Error> {
        Ok(())
    }
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

#[test]
fn invalid_bearer_token_returns_401() {
    let mut container = ServiceContainer::new();
    container.register_instance::<Arc<dyn TokenService>>(Arc::new(RejectingTokenService));
    let mut context = HttpContext::new(
        HttpRequest::new("GET", "/secure"),
        container.build(),
        ConfigBuilder::new().build(),
    );
    context
        .request_mut()
        .headers_mut()
        .insert("authorization", "Bearer expired-token");

    run_middleware(&mut context);

    assert_eq!(context.response().status(), 401);
}
