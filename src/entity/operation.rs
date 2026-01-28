use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::marker::PhantomData;
use std::str::FromStr;
use std::sync::Arc;

use crate::data::paging::PageRequest;
use crate::data::provider::DataProvider;
use crate::endpoint::http_handler::HttpHandler;
use crate::entity::entity::Entity;
use crate::entity::hooks::EntityHooks;
use crate::http::context::HttpContext;
use crate::pipeline::pipeline::PipelineError;
use crate::result::into_response::ResponseValue;
use crate::result::Json;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityOperation {
    List,
    Get,
    Create,
    Update,
    Delete,
}

impl EntityOperation {
    pub fn all() -> &'static [Self] {
        &[
            Self::List,
            Self::Get,
            Self::Create,
            Self::Update,
            Self::Delete,
        ]
    }
}

pub struct OperationHandler<E, H>
where
    E: Entity,
    H: EntityHooks<E>,
{
    operation: EntityOperation,
    hooks: Arc<H>,
    _entity: PhantomData<E>,
}

impl<E, H> OperationHandler<E, H>
where
    E: Entity,
    H: EntityHooks<E>,
{
    pub fn new(operation: EntityOperation, hooks: Arc<H>) -> Self {
        Self {
            operation,
            hooks,
            _entity: PhantomData,
        }
    }
}

#[async_trait]
impl<E, H> HttpHandler for OperationHandler<E, H>
where
    E: Entity + Serialize + DeserializeOwned + 'static,
    E::Id: FromStr + Send + Sync + 'static,
    H: EntityHooks<E> + 'static,
{
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let provider = context
            .services()
            .resolve::<Arc<dyn DataProvider<E>>>()
            .ok_or_else(|| {
                PipelineError::message(&format!("DataProvider<{}> not found", E::name()))
            })?;

        match self.operation {
            EntityOperation::List => {
                let page = context
                    .route()
                    .and_then(|r| r.params().get("page"))
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(1);
                let page_size = context
                    .route()
                    .and_then(|r| r.params().get("pageSize"))
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(20);

                let request = PageRequest::new(page, page_size);
                let result = provider
                    .list(request)
                    .await
                    .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;
                Ok(ResponseValue::new(Json(result)))
            }
            EntityOperation::Get => {
                let id_str = context
                    .route()
                    .and_then(|r| r.params().get("id"))
                    .ok_or_else(|| PipelineError::message("id parameter missing"))?;

                let id = E::Id::from_str(id_str)
                    .map_err(|_| PipelineError::message("invalid id format"))?;

                let result = provider
                    .get(&id)
                    .await
                    .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

                match result {
                    Some(entity) => Ok(ResponseValue::new(Json(entity))),
                    None => Err(PipelineError::message("not found")),
                }
            }
            EntityOperation::Create => {
                let mut entity: E = context
                    .read_json()
                    .map_err(|e| PipelineError::message(e.message()))?;

                self.hooks
                    .before_insert(context, &mut entity)
                    .await
                    .map_err(|e| PipelineError::message(&e.to_string()))?;

                let result = provider
                    .create(entity)
                    .await
                    .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

                self.hooks
                    .after_insert(context, &result)
                    .await
                    .map_err(|e| PipelineError::message(&e.to_string()))?;

                Ok(ResponseValue::new(Json(result)))
            }
            EntityOperation::Update => {
                let mut entity: E = context
                    .read_json()
                    .map_err(|e| PipelineError::message(e.message()))?;

                self.hooks
                    .before_update(context, &mut entity)
                    .await
                    .map_err(|e| PipelineError::message(&e.to_string()))?;

                let result = provider
                    .update(entity)
                    .await
                    .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

                self.hooks
                    .after_update(context, &result)
                    .await
                    .map_err(|e| PipelineError::message(&e.to_string()))?;

                Ok(ResponseValue::new(Json(result)))
            }
            EntityOperation::Delete => {
                let id_str = context
                    .route()
                    .and_then(|r| r.params().get("id"))
                    .ok_or_else(|| PipelineError::message("id parameter missing"))?;

                self.hooks
                    .before_delete(context, id_str)
                    .await
                    .map_err(|e| PipelineError::message(&e.to_string()))?;

                let id = E::Id::from_str(id_str)
                    .map_err(|_| PipelineError::message("invalid id format"))?;

                let success = provider
                    .delete(&id)
                    .await
                    .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

                if success {
                    self.hooks
                        .after_delete(context, id_str)
                        .await
                        .map_err(|e| PipelineError::message(&e.to_string()))?;
                    Ok(ResponseValue::new("deleted"))
                } else {
                    Err(PipelineError::message("not found or could not delete"))
                }
            }
        }
    }
}
