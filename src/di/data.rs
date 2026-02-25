use std::sync::Arc;

use async_trait::async_trait;

use crate::data::provider::DataProvider;
use crate::data::repository::Repository;
use crate::entity::entity::Entity;

pub trait DataProviderRegistry {
    fn add_data_provider<E, P>(&mut self, provider: P) -> &mut Self
    where
        E: Entity,
        P: DataProvider<E> + 'static;

    fn add_data_provider_shared<E>(&mut self, provider: Arc<dyn DataProvider<E>>) -> &mut Self
    where
        E: Entity;
}

impl DataProviderRegistry for crate::di::container::ServiceContainer {
    fn add_data_provider<E, P>(&mut self, provider: P) -> &mut Self
    where
        E: Entity,
        P: DataProvider<E> + 'static,
    {
        let provider = Arc::new(provider) as Arc<dyn DataProvider<E>>;
        self.add_data_provider_shared::<E>(provider)
    }

    fn add_data_provider_shared<E>(&mut self, provider: Arc<dyn DataProvider<E>>) -> &mut Self
    where
        E: Entity,
    {
        log::debug!(
            "Registering provider for {} ({})",
            E::name(),
            E::plural_name()
        );

        let provider_clone = Arc::clone(&provider);
        self.register_singleton::<Arc<dyn DataProvider<E>>, _>(move |_| {
            Arc::clone(&provider_clone)
        });

        self.register_singleton::<Repository<E>, _>(move |services| {
            let provider = services
                .resolve::<Arc<dyn DataProvider<E>>>()
                .expect("data provider not registered");
            let wrapped = SharedProvider {
                inner: (*provider).clone(),
            };
            Repository::new(Box::new(wrapped))
        });

        self
    }
}

struct SharedProvider<E: Entity> {
    inner: Arc<dyn DataProvider<E>>,
}

#[async_trait]
impl<E: Entity> DataProvider<E> for SharedProvider<E> {
    async fn create(&self, entity: E) -> crate::data::provider::DataResult<E> {
        self.inner.create(entity).await
    }

    async fn get(&self, id: &E::Id) -> crate::data::provider::DataResult<Option<E>> {
        self.inner.get(id).await
    }

    async fn update(&self, entity: E) -> crate::data::provider::DataResult<E> {
        self.inner.update(entity).await
    }

    async fn delete(&self, id: &E::Id) -> crate::data::provider::DataResult<bool> {
        self.inner.delete(id).await
    }

    async fn query(
        &self,
        query: crate::data::query::Query<E>,
    ) -> crate::data::provider::DataResult<crate::data::paging::Page<E>> {
        self.inner.query(query).await
    }

    async fn get_by(
        &self,
        column: &str,
        value: crate::data::query::Value,
    ) -> crate::data::provider::DataResult<Option<E>> {
        self.inner.get_by(column, value).await
    }

    async fn all(
        &self,
        query: crate::data::query::Query<E>,
    ) -> crate::data::provider::DataResult<Vec<E>> {
        self.inner.all(query).await
    }
}
