use async_trait::async_trait;

use crate::data::paging::Page;
use crate::data::provider::{DataProvider, DataResult};
use crate::data::query::Query;
use crate::data::query::Value;
use crate::entity::entity::Entity;

pub struct Repository<E>
where
    E: Entity,
{
    provider: Box<dyn DataProvider<E>>,
}

impl<E: Entity> Repository<E> {
    pub fn new(provider: Box<dyn DataProvider<E>>) -> Self {
        Self { provider }
    }

    pub fn entity_name(&self) -> &'static str {
        E::name()
    }

    pub fn entity_plural_name(&self) -> String {
        E::plural_name()
    }

    pub fn into_provider(self) -> Box<dyn DataProvider<E>> {
        self.provider
    }

    pub async fn insert(&self, entity: E) -> DataResult<E> {
        self.provider.create(entity).await
    }
}

#[async_trait]
impl<E: Entity> DataProvider<E> for Repository<E> {
    async fn create(&self, entity: E) -> DataResult<E> {
        log::debug!("Repository create {}", E::name());
        self.provider.create(entity).await
    }

    async fn get(&self, id: &E::Id) -> DataResult<Option<E>> {
        log::debug!("Repository get {}", E::name());
        self.provider.get(id).await
    }

    async fn update(&self, entity: E) -> DataResult<E> {
        log::debug!("Repository update {}", E::name());
        self.provider.update(entity).await
    }

    async fn delete(&self, id: &E::Id) -> DataResult<bool> {
        log::debug!("Repository delete {}", E::name());
        self.provider.delete(id).await
    }

    async fn query(&self, query: Query<E>) -> DataResult<Page<E>> {
        log::debug!("Repository query {} ({})", E::name(), E::plural_name());
        self.provider.query(query).await
    }

    async fn get_by(&self, column: &str, value: Value) -> DataResult<Option<E>> {
        log::debug!("Repository get_by {} column={}", E::name(), column);
        self.provider.get_by(column, value).await
    }
}
