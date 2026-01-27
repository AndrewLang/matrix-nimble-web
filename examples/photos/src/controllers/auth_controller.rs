use std::sync::Arc;

use nimble_web::controller::controller::Controller;
use nimble_web::controller::registry::ControllerRegistry;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::http::context::HttpContext;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::{Json, ResponseValue};
use nimble_web::security::policy::Policy;

use crate::dtos::auth_dtos::{
    AuthResponse, LoginRequest, LogoutResponse, MeResponse, RegisterRequest,
};
use crate::services::auth_service::{AuthError, AuthService};

pub struct AuthController;

impl Controller for AuthController {
    fn register(registry: &mut ControllerRegistry) {
        registry.post("/api/auth/register", RegisterHandler);
        registry.post("/api/auth/login", LoginHandler);
        registry.post("/api/auth/logout", LogoutHandler);
        registry.add_with_policy("GET", "/api/auth/me", MeHandler, Policy::Authenticated);
    }
}

struct RegisterHandler;

impl HttpHandler for RegisterHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let payload: RegisterRequest = context
            .read_json()
            .map_err(|err| PipelineError::message(err.message()))?;

        let service = resolve_service(context)?;
        let response = service
            .register_user(payload)
            .await
            .map_err(map_auth_error)?;

        context.response_mut().set_status(201);
        Ok(ResponseValue::new(Json(response)))
    }
}

struct LoginHandler;

impl HttpHandler for LoginHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let payload: LoginRequest = context
            .read_json()
            .map_err(|err| PipelineError::message(err.message()))?;

        let service = resolve_service(context)?;
        let response = service
            .login_user(payload)
            .await
            .map_err(map_auth_error)?;

        Ok(ResponseValue::new(Json(response)))
    }
}

struct LogoutHandler;

impl HttpHandler for LogoutHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let user_id = extract_user_id(context)?;
        let service = resolve_service(context)?;
        let response = service
            .logout_user(&user_id)
            .await
            .map_err(map_auth_error)?;

        Ok(ResponseValue::new(Json(response)))
    }
}

struct MeHandler;

impl HttpHandler for MeHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let user_id = extract_user_id(context)?;
        let service = resolve_service(context)?;
        let response = service
            .get_current_user(&user_id)
            .await
            .map_err(map_auth_error)?;

        Ok(ResponseValue::new(Json(response)))
    }
}

fn resolve_service(context: &HttpContext) -> Result<Arc<AuthService>, PipelineError> {
    context
        .services()
        .resolve::<AuthService>()
        .ok_or_else(|| PipelineError::message("auth service is unavailable"))
}

fn extract_user_id(context: &HttpContext) -> Result<String, PipelineError> {
    context
        .request()
        .headers()
        .get("x-user-id")
        .map(str::to_string)
        .ok_or_else(|| PipelineError::message("missing x-user-id header"))
}

fn map_auth_error(error: AuthError) -> PipelineError {
    PipelineError::message(error.message())
}
