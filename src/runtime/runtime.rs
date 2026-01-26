use std::future::Future;
use std::net::TcpListener;
use std::pin::Pin;
use std::sync::Arc;

use crate::app::application::AppError;
use crate::app::application::Application;
pub(crate) type RuntimeFuture<'a> = Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>>;

pub(crate) trait Runtime: Send + Sync {
    async fn run<'a>(
        &'a self,
        listener: TcpListener,
        app: Arc<Application>,
        shutdown: Pin<Box<dyn Future<Output = ()> + Send + 'a>>,
    ) -> Result<(), AppError>;
}
