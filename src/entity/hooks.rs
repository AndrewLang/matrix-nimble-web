use crate::http::context::HttpContext;
use crate::result::Result;
use async_trait::async_trait;

pub type RequestContext = HttpContext;

#[async_trait]
pub trait EntityHooks<T>: Send + Sync {
    async fn before_insert(&self, _context: &RequestContext, _entity: &mut T) -> Result<()> {
        Ok(())
    }

    async fn before_update(&self, _context: &RequestContext, _entity: &mut T) -> Result<()> {
        Ok(())
    }

    async fn before_delete(&self, _context: &RequestContext, _id: &str) -> Result<()> {
        Ok(())
    }

    async fn after_insert(&self, _context: &RequestContext, _entity: &T) -> Result<()> {
        Ok(())
    }

    async fn after_update(&self, _context: &RequestContext, _entity: &T) -> Result<()> {
        Ok(())
    }

    async fn after_delete(&self, _context: &RequestContext, _id: &str) -> Result<()> {
        Ok(())
    }
}

pub struct DefaultEntityHooks;

#[async_trait]
impl<T: Send + Sync> EntityHooks<T> for DefaultEntityHooks {}
