use async_trait::async_trait;

use crate::data::paging::{Page, PageRequest};
use crate::data::query::Query;
use crate::entity::entity::Entity;

#[derive(Debug, Clone)]
pub enum DataError {
    NotFound,
    Conflict(String),
    InvalidQuery(String),
    Provider(String),
}

pub type DataResult<T> = Result<T, DataError>;

#[async_trait]
pub trait DataProvider<E: Entity>: Send + Sync {
    async fn create(&self, entity: E) -> DataResult<E>;

    async fn get(&self, id: &E::Id) -> DataResult<Option<E>>;

    async fn update(&self, entity: E) -> DataResult<E>;

    async fn delete(&self, id: &E::Id) -> DataResult<bool>;

    async fn query(&self, query: Query<E>) -> DataResult<Page<E>>;

    async fn get_by(&self, column: &str, value: crate::data::query::Value)
        -> DataResult<Option<E>>;

    async fn list(&self, page: PageRequest) -> DataResult<Page<E>> {
        let mut query = Query::<E>::new();
        query.paging = Some(page);
        self.query(query).await
    }

    async fn all(&self, query: Query<E>) -> DataResult<Vec<E>>;
}
