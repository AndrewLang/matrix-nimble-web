use std::sync::Arc;

use crate::di::provider::ServiceProvider;

pub struct ServiceScope {
    pub(crate) provider: ServiceProvider,
}

impl ServiceScope {
    pub fn resolve<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.provider.resolve::<T>()
    }
}
