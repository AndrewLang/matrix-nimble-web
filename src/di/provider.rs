use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::di::lifetime::ServiceLifetime;
use crate::di::registration::Registration;
use crate::di::service_scope::ServiceScope;

pub struct ServiceProvider {
    registrations: Arc<HashMap<TypeId, Registration>>,
    singletons: Arc<Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
    scoped: Arc<Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
}

impl Default for ServiceProvider {
    fn default() -> Self {
        Self::from_registrations(HashMap::new())
    }
}

impl ServiceProvider {
    pub(crate) fn from_registrations(registrations: HashMap<TypeId, Registration>) -> Self {
        Self {
            registrations: Arc::new(registrations),
            singletons: Arc::new(Mutex::new(HashMap::new())),
            scoped: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn resolve<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        let registration = self.registrations.get(&TypeId::of::<T>()).cloned()?;

        match registration.lifetime {
            ServiceLifetime::Singleton => self.resolve_cached(&registration, &self.singletons),
            ServiceLifetime::Scoped => self.resolve_cached(&registration, &self.scoped),
            ServiceLifetime::Transient => self.resolve_transient(&registration),
        }
    }

    pub fn create_scope(&self) -> ServiceScope {
        ServiceScope {
            provider: ServiceProvider {
                registrations: Arc::clone(&self.registrations),
                singletons: Arc::clone(&self.singletons),
                scoped: Arc::new(Mutex::new(HashMap::new())),
            },
        }
    }

    fn resolve_cached<T>(
        &self,
        registration: &Registration,
        cache: &Arc<Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
    ) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();

        if let Some(existing) = cache.lock().expect("cache poisoned").get(&type_id) {
            return existing.clone().downcast::<T>().ok();
        }

        let created = (registration.factory)(self);
        let _typed = created.clone().downcast::<T>().ok()?;

        let mut guard = cache.lock().expect("cache poisoned");
        let entry = guard.entry(type_id).or_insert(created);

        entry.clone().downcast::<T>().ok()
    }

    fn resolve_transient<T>(&self, registration: &Registration) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        (registration.factory)(self).downcast::<T>().ok()
    }
}
