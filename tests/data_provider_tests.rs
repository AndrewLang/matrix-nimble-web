use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use nimble_web::data::paging::Page;
use nimble_web::data::provider::{DataProvider, DataResult};
use nimble_web::data::query::{FilterOperator, Query, Value};
use nimble_web::data::query_builder::QueryBuilder;
use nimble_web::data::repository::Repository;
use nimble_web::entity::entity::Entity;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Photo {
    id: i64,
    name: String,
    is_public: bool,
}

impl Entity for Photo {
    type Id = i64;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "photo"
    }

    fn plural_name() -> String {
        "photos".to_string()
    }
}

struct MockDataProvider<E: Entity>
where
    E::Id: Eq + Hash + Clone,
    E: Clone,
{
    store: Arc<Mutex<HashMap<E::Id, E>>>,
}

impl<E> MockDataProvider<E>
where
    E: Entity + Clone,
    E::Id: Eq + Hash + Clone,
{
    fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl<E> DataProvider<E> for MockDataProvider<E>
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
        Ok(self
            .store
            .lock()
            .expect("store lock")
            .remove(id)
            .is_some())
    }

    async fn query(&self, query: Query<E>) -> DataResult<Page<E>> {
        let values: Vec<E> = self.store.lock().expect("store lock").values().cloned().collect();
        let total = values.len() as u64;
        let (page, page_size) = query
            .paging
            .map(|p| (p.page, p.page_size))
            .unwrap_or((1, values.len() as u32));
        Ok(Page::new(values, total, page, page_size))
    }
}

#[tokio::test]
async fn query_results_and_paging_metadata() {
    let provider = MockDataProvider::<Photo>::new();
    let repo = Repository::new(Box::new(provider));

    let _ = repo
        .insert(Photo {
            id: 1,
            name: "one".to_string(),
            is_public: true,
        })
        .await
        .unwrap();
    let _ = repo
        .insert(Photo {
            id: 2,
            name: "two".to_string(),
            is_public: false,
        })
        .await
        .unwrap();

    let query = QueryBuilder::<Photo>::new()
        .filter("is_public", FilterOperator::Eq, Value::Bool(true))
        .page(1, 1)
        .build();

    let page = repo.query(query).await.unwrap();
    assert_eq!(page.total, 2);
    assert_eq!(page.page, 1);
    assert_eq!(page.page_size, 1);
    assert_eq!(page.items.len(), 2);
}

#[test]
fn filter_is_preserved_in_query_builder() {
    let query = QueryBuilder::<Photo>::new()
        .filter("is_public", FilterOperator::Eq, Value::Bool(true))
        .build();

    assert_eq!(query.filters.len(), 1);
    assert_eq!(query.filters[0].field, "is_public");
    assert_eq!(query.filters[0].operator, FilterOperator::Eq);
    assert_eq!(query.filters[0].value, Value::Bool(true));
}
