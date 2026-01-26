use std::sync::Arc;

use crate::http::context::HttpContext;
use crate::identity::claims::Claims;
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

impl Middleware for AuthenticationMiddleware {
    async fn handle(&self, context: &mut HttpContext, next: Next<'_>) -> Result<(), PipelineError> {
        let identity = if let Some(header) = context.request().headers().get("authorization") {
            if let Some(token) = header.strip_prefix("Bearer ") {
                Arc::new(UserIdentity::new(token, Claims::new()))
                    as Arc<dyn crate::identity::identity::Identity>
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
