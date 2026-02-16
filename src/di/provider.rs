use std::any::type_name;
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};

use crate::di::lifetime::ServiceLifetime;
use crate::di::registration::Registration;
use crate::di::service_scope::ServiceScope;

thread_local! {
    static RESOLVE_STACK: RefCell<Vec<TypeId>> = RefCell::new(Vec::new());
    static RESOLVE_NAME_STACK: RefCell<Vec<&'static str>> = RefCell::new(Vec::new());
}

pub struct ServiceProvider {
    registrations: Arc<HashMap<TypeId, Registration>>,
    singletons: Arc<Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
    scoped: Arc<Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
}

impl Clone for ServiceProvider {
    fn clone(&self) -> Self {
        Self {
            registrations: Arc::clone(&self.registrations),
            singletons: Arc::clone(&self.singletons),
            scoped: Arc::new(Mutex::new(HashMap::new())),
        }
    }
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
        let type_id = TypeId::of::<T>();
        let type_name = Self::short_type_name::<T>();

        if RESOLVE_STACK.with(|stack| stack.borrow().contains(&type_id)) {
            let cycle = RESOLVE_NAME_STACK.with(|names| {
                let mut names = names.borrow().clone();
                names.push(type_name);
                names.join(" -> ")
            });
            panic!(
                "Cycle detected while resolving {}. Resolution path: {}",
                type_name, cycle
            );
        }

        RESOLVE_NAME_STACK.with(|names| names.borrow_mut().push(type_name));
        push_resolving(type_id);

        let result = catch_unwind(AssertUnwindSafe(|| {
            let registration = self.registrations.get(&type_id).cloned()?;

            match registration.lifetime {
                ServiceLifetime::Singleton => self.resolve_cached(&registration, &self.singletons),
                ServiceLifetime::Scoped => self.resolve_cached(&registration, &self.scoped),
                ServiceLifetime::Transient => self.resolve_transient(&registration),
            }
        }));

        pop_resolving();
        RESOLVE_NAME_STACK.with(|names| {
            let mut names = names.borrow_mut();
            names.pop();
        });

        match result {
            Ok(value) => value,
            Err(err) => resume_unwind(err),
        }
    }

    pub fn get<T>(&self) -> Arc<T>
    where
        T: Send + Sync + 'static,
    {
        self.resolve::<T>().unwrap_or_else(|| {
            panic!(
                "Service `{}` is not registered",
                Self::short_type_name::<T>()
            )
        })
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

        if let Some(existing) = {
            let guard = cache.lock().expect("cache poisoned");
            guard.get(&type_id).cloned()
        } {
            return existing.downcast::<T>().ok();
        }

        let created = (registration.factory)(self.as_arc());

        let stored = {
            let mut guard = cache.lock().expect("cache poisoned");
            guard
                .entry(type_id)
                .or_insert_with(|| created.clone())
                .clone()
        };

        stored.downcast::<T>().ok()
    }

    fn resolve_transient<T>(&self, registration: &Registration) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        (registration.factory)(self.as_arc()).downcast::<T>().ok()
    }

    fn short_type_name<T>() -> &'static str {
        type_name::<T>()
            .rsplit("::")
            .next()
            .unwrap_or(type_name::<T>())
    }

    fn as_arc(&self) -> Arc<ServiceProvider> {
        Arc::new(ServiceProvider {
            registrations: Arc::clone(&self.registrations),
            singletons: Arc::clone(&self.singletons),
            scoped: Arc::clone(&self.scoped),
        })
    }
}

fn push_resolving(type_id: TypeId) {
    RESOLVE_STACK.with(|stack| stack.borrow_mut().push(type_id));
}

fn pop_resolving() {
    RESOLVE_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        stack.pop().expect("resolve stack underflow");
    });
}
