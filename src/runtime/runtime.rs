use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

use crate::app::application::AppError;
use crate::app::application::Application;

pub(crate) trait Runtime: Send + Sync {
    async fn run<'a>(
        &'a self,
        addr: SocketAddr,
        app: Arc<Application>,
        shutdown: Pin<Box<dyn Future<Output = ()> + Send + 'a>>,
        wants_random: bool,
    ) -> Result<(), AppError>;
}
