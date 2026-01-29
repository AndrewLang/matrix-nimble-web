use nimble_web::identity::claims::Claims;
use nimble_web::identity::user::UserIdentity;
use nimble_web::security::token::{JwtTokenService, TokenService};

#[test]
fn test_create_and_validate_access_token() {
    let secret = "test-secret".to_string();
    let issuer = "test-issuer".to_string();
    let service = JwtTokenService::new(secret, issuer);

    let mut claims = Claims::new();
    claims = claims.add_role("admin");
    let user = UserIdentity::new("user-123", claims);

    let token = service.create_access_token(&user).expect("create token");
    let validated_claims = service
        .validate_access_token(&token)
        .expect("validate token");

    assert_eq!(validated_claims.sub, Some("user-123".to_string()));
    assert_eq!(validated_claims.roles().len(), 1);
    assert!(validated_claims.roles().contains("admin"));
    assert_eq!(validated_claims.iss, Some("test-issuer".to_string()));
}

#[test]
fn test_create_and_validate_refresh_token() {
    let secret = "test-secret".to_string();
    let issuer = "test-issuer".to_string();
    let service = JwtTokenService::new(secret, issuer);

    let token = service
        .create_refresh_token("user-456")
        .expect("create refresh token");
    let user_id = service
        .validate_refresh_token(&token)
        .expect("validate refresh token");

    assert_eq!(user_id, "user-456");
}

#[test]
fn test_invalid_token() {
    let service = JwtTokenService::new("secret".into(), "issuer".into());
    let result = service.validate_access_token("invalid-token");
    assert!(result.is_err());
}

#[test]
fn test_expired_token() {
    let service = JwtTokenService::new("secret".into(), "issuer".into()).with_expiration(-10, 7); // Expired 10 minutes ago

    let user = UserIdentity::new("user-1", Claims::new());
    let token = service.create_access_token(&user).unwrap();

    let result = service.validate_access_token(&token);
    assert!(result.is_err());
}
