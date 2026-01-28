use nimble_web::prelude::*;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut builder = AppBuilder::new();
    
    builder
        .use_address("0.0.0.0:8080")
        .route_get("/hello", HelloHandler);

    let app = builder.build();
    log::info!("Starting test-app at http://0.0.0.0:8080");
    app.start().await?;

    Ok(())
}

struct HelloHandler;

#[async_trait]
impl HttpHandler for HelloHandler {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        Ok(ResponseValue::new("Hello from Nimble!"))
    }
}
