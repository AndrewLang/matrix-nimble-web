use std::fmt;
use std::sync::Arc;

use crate::identity::identity::Identity;

#[derive(Clone)]
pub struct IdentityContext {
    inner: Arc<dyn Identity>,
}

impl IdentityContext {
    pub fn new(identity: Arc<dyn Identity>) -> Self {
        Self { inner: identity }
    }

    pub fn identity(&self) -> Arc<dyn Identity> {
        Arc::clone(&self.inner)
    }

    pub fn is_authenticated(&self) -> bool {
        self.inner.is_authenticated()
    }
}

impl fmt::Debug for IdentityContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IdentityContext")
            .field("subject", &self.inner.subject())
            .field("kind", &self.inner.kind())
            .finish()
    }
}
