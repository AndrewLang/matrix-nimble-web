use crate::di::ServiceContainer;
use crate::di::ServiceProvider;

pub struct TestServices {
    container: ServiceContainer,
}

impl TestServices {
    pub fn new() -> Self {
        Self {
            container: ServiceContainer::new(),
        }
    }

    pub fn add_singleton<T>(mut self, value: T) -> Self
    where
        T: Send + Sync + Clone + 'static,
    {
        let value = std::sync::Arc::new(value);
        self.container
            .register_singleton::<T, _>(move |_| (*value).clone());
        self
    }

    pub fn override_singleton<T>(mut self, value: T) -> Self
    where
        T: Send + Sync + Clone + 'static,
    {
        let value = std::sync::Arc::new(value);
        self.container
            .register_singleton::<T, _>(move |_| (*value).clone());
        self
    }

    pub fn build(self) -> ServiceProvider {
        self.container.build()
    }
}
