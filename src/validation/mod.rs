use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use crate::http::context::HttpContext;

mod error;
mod middleware;
mod validator;

pub use error::ValidationError;
pub use middleware::ValidationMiddleware;
pub use validator::Validator;

pub trait AnyValidator: Send + Sync {
    fn validate(&self, context: &HttpContext) -> Result<(), ValidationError>;
}

#[derive(Clone)]
pub struct ContextValidator {
    func: Arc<dyn Fn(&HttpContext) -> Result<(), ValidationError> + Send + Sync>,
}

impl ContextValidator {
    pub fn new<F>(func: F) -> Self
    where
        F: Fn(&HttpContext) -> Result<(), ValidationError> + Send + Sync + 'static,
    {
        Self {
            func: Arc::new(func),
        }
    }

    pub fn validate(&self, context: &HttpContext) -> Result<(), ValidationError> {
        (self.func)(context)
    }
}

impl Debug for ContextValidator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Validator(..)")
    }
}

impl AnyValidator for ContextValidator {
    fn validate(&self, context: &HttpContext) -> Result<(), ValidationError> {
        ContextValidator::validate(self, context)
    }
}
