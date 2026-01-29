use crate::http::context::HttpContext;
use crate::http::request::HttpRequest;
use crate::identity::claims::Claims;
use crate::identity::context::IdentityContext;
use crate::security::token::TokenService;
use std::sync::Arc;

pub trait WithAuth {
    fn with_auth(self, user_id: &str) -> Self;
}

impl WithAuth for HttpRequest {
    fn with_auth(mut self, user_id: &str) -> Self {
        let value = format!("Bearer {}", user_id);
        self.headers_mut().insert("authorization", &value);
        self
    }
}

pub struct MockTokenService;

impl TokenService for MockTokenService {
    fn create_access_token(
        &self,
        user: &crate::identity::user::UserIdentity,
    ) -> std::result::Result<String, anyhow::Error> {
        Ok(user.id().to_string())
    }

    fn validate_access_token(&self, token: &str) -> std::result::Result<Claims, anyhow::Error> {
        let mut claims = Claims::new();
        claims.sub = Some(token.to_string());
        Ok(claims)
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

pub fn assert_authenticated(context: &HttpContext) {
    assert!(
        context
            .get::<IdentityContext>()
            .map_or(false, |identity| identity.is_authenticated()),
        "expected authenticated user in context"
    );
}

pub fn provide_mock_token_service(container: &mut crate::di::ServiceContainer) {
    container.register_instance::<Arc<dyn TokenService>>(Arc::new(MockTokenService));
}
