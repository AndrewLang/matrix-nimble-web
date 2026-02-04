use crate::http::context::HttpContext;
use crate::identity::context::IdentityContext;
use crate::pipeline::middleware::Middleware;
use crate::pipeline::next::Next;
use crate::pipeline::pipeline::PipelineError;
use async_trait::async_trait;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Policy {
    Authenticated,
    InRole(String),
    Custom(String),
}

impl Policy {
    pub fn allows(&self, ctx: &HttpContext) -> bool {
        match self {
            Policy::Authenticated => ctx
                .get::<IdentityContext>()
                .map_or(false, |identity| identity.is_authenticated()),
            Policy::InRole(role) => ctx.get::<IdentityContext>().map_or(false, |identity| {
                identity.is_authenticated() && identity.identity().claims().roles().contains(role)
            }),
            Policy::Custom(_) => false,
        }
    }
}

pub struct AuthorizationMiddleware;

impl AuthorizationMiddleware {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Middleware for AuthorizationMiddleware {
    async fn handle(&self, context: &mut HttpContext, next: Next<'_>) -> Result<(), PipelineError> {
        if let Some(endpoint) = context.endpoint() {
            if let Some(policy) = endpoint.metadata().policy() {
                if !policy.allows(context) {
                    context.response_mut().set_status(403);
                    return Ok(());
                }
            }
        }

        next.run(context).await
    }
}
