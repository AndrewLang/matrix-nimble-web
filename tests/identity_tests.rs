use std::sync::Arc;

use nimble_web::identity::claims::Claims;
use nimble_web::identity::context::IdentityContext;
use nimble_web::identity::identity::Identity;
use nimble_web::identity::kind::IdentityKind;
use nimble_web::identity::method::AuthMethod;
use nimble_web::identity::user::{AnonymousIdentity, UserIdentity};

#[test]
fn claims_builder_round_trips_collections() {
    let claims = Claims::new()
        .add_role("admin")
        .add_permission("read")
        .add_attribute("department", "engineering");

    assert!(claims.roles().contains("admin"));
    assert!(claims.permissions().contains("read"));
    assert_eq!(
        claims.attributes().get("department").map(String::as_str),
        Some("engineering")
    );
}

#[test]
fn user_identity_returns_expected_metadata() {
    let claims = Claims::new().add_role("user");
    let identity = UserIdentity::new("user-1", claims.clone());

    assert_eq!(identity.subject(), "user-1");
    assert_eq!(identity.kind(), IdentityKind::User);
    assert_eq!(identity.auth_method(), AuthMethod::Bearer);
    assert_eq!(identity.claims(), &claims);
    assert!(identity.is_authenticated());
}

#[test]
fn anonymous_identity_reports_unauthenticated() {
    let identity = AnonymousIdentity::new();

    assert_eq!(identity.subject(), "anonymous");
    assert_eq!(identity.kind(), IdentityKind::Anonymous);
    assert_eq!(identity.auth_method(), AuthMethod::Anonymous);
    assert!(identity.claims().roles().is_empty());
    assert!(!identity.is_authenticated());
}

#[test]
fn identity_context_exposes_identity_and_auth_status() {
    let identity = Arc::new(UserIdentity::new("uid", Claims::new()));
    let context = IdentityContext::new(identity.clone());

    assert!(context.is_authenticated());
    let from_context = context.identity();
    assert_eq!(from_context.subject(), "uid");
}
