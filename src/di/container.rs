use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use crate::di::lifetime::ServiceLifetime;
use crate::di::provider::ServiceProvider;
use crate::di::registration::Registration;

#[derive(Default)]
pub struct ServiceContainer {
    registrations: HashMap<TypeId, Registration>,
}

impl ServiceContainer {
    pub fn new() -> Self {
        Self {
            registrations: HashMap::new(),
        }
    }

    pub fn register<T, F>(&mut self, lifetime: ServiceLifetime, factory: F)
    where
        T: Send + Sync + 'static,
        F: Fn(&ServiceProvider) -> T + Send + Sync + 'static,
    {
        let factory = Arc::new(move |provider: &ServiceProvider| {
            let value = factory(provider);
            Arc::new(value) as Arc<dyn Any + Send + Sync>
        });

        self.registrations
            .insert(TypeId::of::<T>(), Registration::new(lifetime, factory));
    }

    pub fn register_singleton<T, F>(&mut self, factory: F)
    where
        T: Send + Sync + 'static,
        F: Fn(&ServiceProvider) -> T + Send + Sync + 'static,
    {
        self.register(ServiceLifetime::Singleton, factory);
    }

    pub fn register_scoped<T, F>(&mut self, factory: F)
    where
        T: Send + Sync + 'static,
        F: Fn(&ServiceProvider) -> T + Send + Sync + 'static,
    {
        self.register(ServiceLifetime::Scoped, factory);
    }

    pub fn register_transient<T, F>(&mut self, factory: F)
    where
        T: Send + Sync + 'static,
        F: Fn(&ServiceProvider) -> T + Send + Sync + 'static,
    {
        self.register(ServiceLifetime::Transient, factory);
    }

    pub fn build(self) -> ServiceProvider {
        ServiceProvider::from_registrations(self.registrations)
    }
}
