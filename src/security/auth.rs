use crate::http::context::HttpContext;
use crate::pipeline::middleware::Middleware;
use crate::pipeline::next::Next;
use crate::pipeline::pipeline::PipelineError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct User {
    pub id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserIdentity {
    pub name: String,
}

pub struct AuthenticationMiddleware;

impl AuthenticationMiddleware {
    pub fn new() -> Self {
        Self
    }
}

impl Middleware for AuthenticationMiddleware {
    async fn handle(&self, context: &mut HttpContext, next: Next<'_>) -> Result<(), PipelineError> {
        if let Some(header) = context.request().headers().get("authorization") {
            if let Some(token) = header.strip_prefix("Bearer ") {
                context.insert(User {
                    id: token.to_string(),
                });
            }
        }
        next.run(context).await
    }
}
