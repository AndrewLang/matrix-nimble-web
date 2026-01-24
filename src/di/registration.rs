use std::any::Any;
use std::sync::Arc;

use crate::di::lifetime::ServiceLifetime;
use crate::di::provider::ServiceProvider;

#[derive(Clone)]
pub(crate) struct Registration {
    pub(crate) lifetime: ServiceLifetime,
    pub(crate) factory: Arc<dyn Fn(&ServiceProvider) -> Arc<dyn Any + Send + Sync> + Send + Sync>,
}

impl Registration {
    pub(crate) fn new(
        lifetime: ServiceLifetime,
        factory: Arc<dyn Fn(&ServiceProvider) -> Arc<dyn Any + Send + Sync> + Send + Sync>,
    ) -> Self {
        Self { lifetime, factory }
    }
}
