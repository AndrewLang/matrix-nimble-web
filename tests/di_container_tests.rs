use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use nimble_web::data::paging::Page;
use nimble_web::data::provider::{DataProvider, DataResult};
use nimble_web::data::query::Query;
use nimble_web::data::repository::Repository;
use nimble_web::di::DataProviderRegistry;
use nimble_web::di::ServiceContainer;
use nimble_web::entity::entity::Entity;

#[derive(Debug, Clone)]
struct Photo {
    id: i64,
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

#[derive(Debug, Clone)]
struct Album {
    id: i64,
}

impl Entity for Album {
    type Id = i64;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "album"
    }

    fn plural_name() -> String {
        "albums".to_string()
    }
}

struct MockProvider<E: Entity>
where
    E::Id: Eq + Hash + Clone,
    E: Clone,
{
    store: Arc<Mutex<HashMap<E::Id, E>>>,
    calls: Arc<Mutex<Vec<&'static str>>>,
    label: &'static str,
}

impl<E> MockProvider<E>
where
    E: Entity + Clone,
    E::Id: Eq + Hash + Clone,
{
    fn new(label: &'static str) -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            calls: Arc::new(Mutex::new(Vec::new())),
            label,
        }
    }
}

#[async_trait::async_trait]
impl<E> DataProvider<E> for MockProvider<E>
where
    E: Entity + Clone + Send + Sync + 'static,
    E::Id: Eq + Hash + Clone + Send + Sync + 'static,
{
    async fn create(&self, entity: E) -> DataResult<E> {
        let _ = self.label;
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
async fn provider_registered_per_entity() {
    let mut container = ServiceContainer::new();
    let photo_provider = MockProvider::<Photo>::new("photos");
    container.add_data_provider::<Photo, _>(photo_provider);

    let services = container.build();
    let resolved = services.resolve::<Arc<dyn DataProvider<Photo>>>();
    assert!(resolved.is_some());
}

#[tokio::test]
async fn repository_resolves_correctly() {
    let mut container = ServiceContainer::new();
    container.add_data_provider::<Photo, _>(MockProvider::<Photo>::new("photos"));

    let services = container.build();
    let repo = services.resolve::<Repository<Photo>>();
    assert!(repo.is_some());

    let repo = repo.expect("repo");
    let entity = Photo { id: 1 };
    let _ = repo.insert(entity).await.unwrap();
}

#[tokio::test]
async fn multiple_entities_do_not_conflict() {
    let mut container = ServiceContainer::new();
    container.add_data_provider::<Photo, _>(MockProvider::<Photo>::new("photos"));
    container.add_data_provider::<Album, _>(MockProvider::<Album>::new("albums"));

    let services = container.build();
    let photo_repo = services.resolve::<Repository<Photo>>();
    let album_repo = services.resolve::<Repository<Album>>();

    assert!(photo_repo.is_some());
    assert!(album_repo.is_some());
}

#[tokio::test]
async fn correct_provider_used_per_entity() {
    let mut container = ServiceContainer::new();
    let photo_provider = MockProvider::<Photo>::new("photos");
    let album_provider = MockProvider::<Album>::new("albums");
    let photo_calls = Arc::clone(&photo_provider.calls);
    let album_calls = Arc::clone(&album_provider.calls);

    container.add_data_provider::<Photo, _>(photo_provider);
    container.add_data_provider::<Album, _>(album_provider);

    let services = container.build();
    let photo_repo = services.resolve::<Repository<Photo>>().expect("photo repo");
    let album_repo = services.resolve::<Repository<Album>>().expect("album repo");

    let _ = photo_repo.insert(Photo { id: 1 }).await.unwrap();
    let _ = album_repo.insert(Album { id: 1 }).await.unwrap();

    let photo_snapshot = photo_calls.lock().expect("calls lock").clone();
    let album_snapshot = album_calls.lock().expect("calls lock").clone();

    assert_eq!(photo_snapshot, vec!["create"]);
    assert_eq!(album_snapshot, vec!["create"]);
}
