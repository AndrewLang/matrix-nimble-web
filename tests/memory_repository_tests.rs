use nimble_web::data::memory_repository::MemoryRepository;
use nimble_web::data::paging::PageRequest;
use nimble_web::data::provider::DataProvider;
use nimble_web::data::query::Query;
use nimble_web::entity::entity::Entity;

#[derive(Debug, Clone, PartialEq, Eq)]
struct MemoryEntity {
    id: i32,
    name: String,
}

impl Entity for MemoryEntity {
    type Id = i32;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "memory_entity"
    }

    fn plural_name() -> String {
        "memory_entities".to_string()
    }
}

#[tokio::test]
async fn memory_repository_supports_basic_crud() {
    let repo = MemoryRepository::<MemoryEntity>::new();
    let entity = MemoryEntity {
        id: 10,
        name: "memory".to_string(),
    };

    let inserted = repo.create(entity.clone()).await.unwrap();
    assert_eq!(inserted, entity);

    let fetched = repo.get(&entity.id).await.unwrap().unwrap();
    assert_eq!(fetched, entity);

    let updated = MemoryEntity {
        id: 10,
        name: "updated".to_string(),
    };
    let updated = repo.update(updated.clone()).await.unwrap();
    assert_eq!(updated, updated.clone());

    let deleted = repo.delete(&10).await.unwrap();
    assert!(deleted);
}

#[tokio::test]
async fn memory_repository_query_applies_paging() {
    let repo = MemoryRepository::<MemoryEntity>::new();
    for id in 1..=5 {
        let _ = repo
            .create(MemoryEntity {
                id,
                name: format!("item-{}", id),
            })
            .await
            .unwrap();
    }

    let mut query = Query::<MemoryEntity>::new();
    query.paging = Some(PageRequest::new(2, 2));

    let page = repo.query(query).await.unwrap();
    assert_eq!(page.page, 2);
    assert_eq!(page.page_size, 2);
    assert_eq!(page.items.len(), 2);
    assert_eq!(page.total, 5);
}
