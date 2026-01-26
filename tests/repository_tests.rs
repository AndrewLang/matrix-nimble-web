use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use nimble_web::data::paging::{Page, PageRequest};
use nimble_web::data::provider::{DataProvider, DataResult};
use nimble_web::data::query::Query;
use nimble_web::data::repository::Repository;
use nimble_web::entity::entity::Entity;

#[derive(Debug, Clone, PartialEq, Eq)]
struct TestEntity {
    id: i64,
    name: String,
}

impl Entity for TestEntity {
    type Id = i64;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "test_entity"
    }

    fn plural_name() -> String {
        "test_entities".to_string()
    }
}

struct MockDataProvider<E: Entity>
where
    E::Id: Eq + Hash + Clone,
    E: Clone,
{
    store: Arc<Mutex<HashMap<E::Id, E>>>,
    calls: Arc<Mutex<Vec<&'static str>>>,
}

impl<E> MockDataProvider<E>
where
    E: Entity + Clone,
    E::Id: Eq + Hash + Clone,
{
    fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            calls: Arc::new(Mutex::new(Vec::new())),
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
        self.calls.lock().expect("calls lock").push("create");
        self.store
            .lock()
            .expect("store lock")
            .insert(entity.id().clone(), entity.clone());
        Ok(entity)
    }

    async fn get(&self, id: &E::Id) -> DataResult<Option<E>> {
        self.calls.lock().expect("calls lock").push("get");
        Ok(self.store.lock().expect("store lock").get(id).cloned())
    }

    async fn update(&self, entity: E) -> DataResult<E> {
        self.calls.lock().expect("calls lock").push("update");
        self.store
            .lock()
            .expect("store lock")
            .insert(entity.id().clone(), entity.clone());
        Ok(entity)
    }

    async fn delete(&self, id: &E::Id) -> DataResult<bool> {
        self.calls.lock().expect("calls lock").push("delete");
        Ok(self
            .store
            .lock()
            .expect("store lock")
            .remove(id)
            .is_some())
    }

    async fn query(&self, query: Query<E>) -> DataResult<Page<E>> {
        self.calls.lock().expect("calls lock").push("query");
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
async fn repository_delegates_calls() {
    let provider = MockDataProvider::<TestEntity>::new();
    let calls = Arc::clone(&provider.calls);
    let repo = Repository::new(Box::new(provider));

    let _ = repo
        .insert(TestEntity {
            id: 1,
            name: "one".to_string(),
        })
        .await
        .unwrap();
    let _ = repo.get(&1).await.unwrap();
    let _ = repo
        .update(TestEntity {
            id: 1,
            name: "two".to_string(),
        })
        .await
        .unwrap();
    let _ = repo.delete(&1).await.unwrap();
    let _ = repo.query(Query::<TestEntity>::new()).await.unwrap();

    let snapshot = calls.lock().expect("calls lock").clone();
    assert_eq!(snapshot, vec!["create", "get", "update", "delete", "query"]);
}

#[test]
fn repository_exposes_entity_names_and_provider() {
    let provider = MockDataProvider::<TestEntity>::new();
    let repo = Repository::new(Box::new(provider));

    assert_eq!(repo.entity_name(), "test_entity");
    assert_eq!(repo.entity_plural_name(), "test_entities".to_string());

    let _ = repo.into_provider();
}

#[tokio::test]
async fn insert_and_get_roundtrip() {
    let provider = MockDataProvider::<TestEntity>::new();
    let repo = Repository::new(Box::new(provider));
    let entity = TestEntity {
        id: 42,
        name: "answer".to_string(),
    };

    let inserted = repo.insert(entity.clone()).await.unwrap();
    let fetched = repo.get(&42).await.unwrap().expect("entity");

    assert_eq!(inserted, entity);
    assert_eq!(fetched, entity);
}

#[tokio::test]
async fn update_modifies_entity() {
    let provider = MockDataProvider::<TestEntity>::new();
    let repo = Repository::new(Box::new(provider));

    let entity = TestEntity {
        id: 7,
        name: "old".to_string(),
    };
    let _ = repo.insert(entity).await.unwrap();

    let updated = repo
        .update(TestEntity {
            id: 7,
            name: "new".to_string(),
        })
        .await
        .unwrap();

    let fetched = repo.get(&7).await.unwrap().expect("entity");
    assert_eq!(updated, fetched);
    assert_eq!(fetched.name, "new");
}

#[tokio::test]
async fn delete_removes_entity() {
    let provider = MockDataProvider::<TestEntity>::new();
    let repo = Repository::new(Box::new(provider));

    let _ = repo
        .insert(TestEntity {
            id: 3,
            name: "temp".to_string(),
        })
        .await
        .unwrap();

    let deleted = repo.delete(&3).await.unwrap();
    let fetched = repo.get(&3).await.unwrap();

    assert!(deleted);
    assert!(fetched.is_none());
}

#[tokio::test]
async fn query_returns_expected_page() {
    let provider = MockDataProvider::<TestEntity>::new();
    let repo = Repository::new(Box::new(provider));

    for id in 1..=3 {
        let _ = repo
            .insert(TestEntity {
                id,
                name: format!("item-{}", id),
            })
            .await
            .unwrap();
    }

    let mut query = Query::<TestEntity>::new();
    query.paging = Some(PageRequest::new(1, 2));

    let page = repo.query(query).await.unwrap();

    assert_eq!(page.total, 3);
    assert_eq!(page.page, 1);
    assert_eq!(page.page_size, 2);
    assert_eq!(page.items.len(), 3);
}
