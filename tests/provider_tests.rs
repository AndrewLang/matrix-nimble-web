use nimble_web::data::paging::{Page, PageRequest};
use nimble_web::data::provider::{DataProvider, DataResult};
use nimble_web::data::query::Query;
use nimble_web::entity::entity::Entity;

#[derive(Debug, Clone)]
struct Thing {
    id: i64,
}

impl Entity for Thing {
    type Id = i64;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "thing"
    }

    fn plural_name() -> String {
        "things".to_string()
    }
}

struct FakeProvider;

#[async_trait::async_trait]
impl DataProvider<Thing> for FakeProvider {
    async fn create(&self, entity: Thing) -> DataResult<Thing> {
        Ok(entity)
    }

    async fn get(&self, _id: &i64) -> DataResult<Option<Thing>> {
        Ok(None)
    }

    async fn update(&self, entity: Thing) -> DataResult<Thing> {
        Ok(entity)
    }

    async fn delete(&self, _id: &i64) -> DataResult<bool> {
        Ok(true)
    }

    async fn query(&self, query: Query<Thing>) -> DataResult<Page<Thing>> {
        let (page, page_size) = query
            .paging
            .map(|p| (p.page, p.page_size))
            .unwrap_or((1, 10));
        Ok(Page::new(Vec::new(), 0, page, page_size))
    }
}

#[tokio::test]
async fn list_applies_paging() {
    let provider = FakeProvider;
    let page = provider.list(PageRequest::new(2, 25)).await.unwrap();
    assert_eq!(page.page, 2);
    assert_eq!(page.page_size, 25);
}
