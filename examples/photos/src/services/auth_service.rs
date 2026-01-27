use chrono::Utc;

use crate::dtos::auth_dtos::{
    AuthResponse, LoginRequest, LogoutResponse, MeResponse, RegisterRequest,
};

#[derive(Clone, Debug)]
pub struct AuthService;

impl AuthService {
    pub fn new() -> Self {
        Self
    }

    pub async fn register_user(
        &self,
        request: RegisterRequest,
    ) -> Result<AuthResponse, AuthError> {
        let id = format!("user-{}", Utc::now().timestamp_millis());
        Ok(AuthResponse {
            id,
            email: request.email,
            display_name: request.display_name,
            token: "mock-token".into(),
        })
    }

    pub async fn login_user(&self, request: LoginRequest) -> Result<AuthResponse, AuthError> {
        Ok(AuthResponse {
            id: format!("user-{}", request.email),
            email: request.email,
            display_name: "Mock User".into(),
            token: "mock-session-token".into(),
        })
    }

    pub async fn logout_user(&self, user_id: &str) -> Result<LogoutResponse, AuthError> {
        Ok(LogoutResponse {
            message: format!("user {} logged out", user_id),
        })
    }

    pub async fn get_current_user(&self, user_id: &str) -> Result<MeResponse, AuthError> {
        Ok(MeResponse {
            id: user_id.to_string(),
            email: format!("{}@example.com", user_id),
            display_name: "Mock User".into(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct AuthError {
    message: String,
}

impl AuthError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}
