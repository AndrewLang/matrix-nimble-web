use async_trait::async_trait;

use crate::data::provider::{DataProvider, DataResult};
use crate::data::query::Query;
use crate::entity::entity::Entity;

pub struct Repository<E: Entity> {
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
        log::debug!("repo.create {}", E::name());
        self.provider.create(entity).await
    }

    async fn get(&self, id: &E::Id) -> DataResult<Option<E>> {
        log::debug!("repo.get {}", E::name());
        self.provider.get(id).await
    }

    async fn update(&self, entity: E) -> DataResult<E> {
        log::debug!("repo.update {}", E::name());
        self.provider.update(entity).await
    }

    async fn delete(&self, id: &E::Id) -> DataResult<bool> {
        log::debug!("repo.delete {}", E::name());
        self.provider.delete(id).await
    }

    async fn query(&self, query: Query<E>) -> DataResult<crate::data::paging::Page<E>> {
        log::debug!(
            "repo.query {} ({})",
            E::name(),
            E::plural_name()
        );
        self.provider.query(query).await
    }
}
