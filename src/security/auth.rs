use async_trait::async_trait;
use std::sync::Arc;

use crate::http::context::HttpContext;
use crate::identity::context::IdentityContext;
use crate::identity::user::{AnonymousIdentity, UserIdentity};
use crate::pipeline::middleware::Middleware;
use crate::pipeline::next::Next;
use crate::pipeline::pipeline::PipelineError;

pub struct AuthenticationMiddleware;

impl AuthenticationMiddleware {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Middleware for AuthenticationMiddleware {
    async fn handle(&self, context: &mut HttpContext, next: Next<'_>) -> Result<(), PipelineError> {
        let identity = if let Some(header) = context.request().headers().get("authorization") {
            if let Some(token) = header.strip_prefix("Bearer ") {
                let token_service = context
                    .services()
                    .resolve::<Arc<dyn crate::security::token::TokenService>>();

                if let Some(service) = token_service {
                    match service.validate_access_token(token) {
                        Ok(claims) => {
                            if let Some(sub) = claims.sub.clone() {
                                Arc::new(UserIdentity::new(sub, claims))
                                    as Arc<dyn crate::identity::identity::Identity>
                            } else {
                                log::warn!("Token missing sub claim");
                                Arc::new(AnonymousIdentity::new())
                            }
                        }
                        Err(err) => {
                            log::warn!("Token validation failed: {}", err);
                            Arc::new(AnonymousIdentity::new())
                        }
                    }
                } else {
                    log::warn!("TokenService not found in container, cannot validate token");
                    Arc::new(AnonymousIdentity::new())
                }
            } else {
                Arc::new(AnonymousIdentity::new())
            }
        } else {
            Arc::new(AnonymousIdentity::new())
        };

        context.insert(IdentityContext::new(identity));
        next.run(context).await
    }
}
