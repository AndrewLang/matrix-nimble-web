use async_trait::async_trait;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use crate::data::paging::Page;
use crate::data::provider::{DataProvider, DataResult};
use crate::data::query::Query;
use crate::data::query::Value;
use crate::entity::entity::Entity;

#[derive(Clone)]
pub struct MemoryRepository<E>
where
    E: Entity + Clone,
    E::Id: Eq + Hash + Clone,
{
    store: Arc<Mutex<HashMap<E::Id, E>>>,
}

impl<E> MemoryRepository<E>
where
    E: Entity + Clone,
    E::Id: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn seed(&self, entities: Vec<E>) {
        let mut store = self.store.lock().expect("store lock");
        for entity in entities {
            store.insert(entity.id().clone(), entity);
        }
    }

    fn snapshot(&self) -> Vec<E> {
        self.store
            .lock()
            .expect("store lock")
            .values()
            .cloned()
            .collect()
    }
}

#[async_trait]
impl<E> DataProvider<E> for MemoryRepository<E>
where
    E: Entity + Clone + Send + Sync + 'static,
    E::Id: Eq + Hash + Clone + Send + Sync + 'static,
{
    async fn create(&self, entity: E) -> DataResult<E> {
        self.store
            .lock()
            .expect("store lock")
            .insert(entity.id().clone(), entity.clone());
        Ok(entity)
    }

    async fn get(&self, id: &E::Id) -> DataResult<Option<E>> {
        Ok(self.store.lock().expect("store lock").get(id).cloned())
    }

    async fn update(&self, entity: E) -> DataResult<E> {
        self.store
            .lock()
            .expect("store lock")
            .insert(entity.id().clone(), entity.clone());
        Ok(entity)
    }

    async fn delete(&self, id: &E::Id) -> DataResult<bool> {
        Ok(self.store.lock().expect("store lock").remove(id).is_some())
    }

    async fn delete_by(&self, column: &str, value: Value) -> DataResult<bool> {
        let _ = (column, value);
        Ok(false)
    }

    async fn query(&self, query: Query<E>) -> DataResult<Page<E>> {
        let mut items = self.snapshot();
        let total = items.len() as u64;
        let (page, page_size) = query
            .paging
            .map(|p| (p.page.max(1), p.page_size.max(1)))
            .unwrap_or((
                1,
                if items.is_empty() {
                    1
                } else {
                    items.len() as u32
                },
            ));

        let start = ((page - 1) as usize).saturating_mul(page_size as usize);
        let end = (start + page_size as usize).min(items.len());
        let page_items = if start < items.len() {
            items.drain(start..end).collect()
        } else {
            Vec::new()
        };

        Ok(Page::new(page_items, total, page, page_size))
    }

    async fn get_by(&self, _column: &str, _value: Value) -> DataResult<Option<E>> {
        // TODO: Implement filtering for MemoryRepository
        Ok(None)
    }

    async fn all(&self, _query: Query<E>) -> DataResult<Vec<E>> {
        Ok(self.snapshot())
    }
}
