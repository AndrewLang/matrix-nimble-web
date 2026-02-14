use async_trait::async_trait;

use crate::http::context::HttpContext;
use crate::http::response_body::ResponseBody;
use crate::pipeline::middleware::Middleware;
use crate::pipeline::next::Next;
use crate::pipeline::pipeline::PipelineError;

const DEFAULT_METHODS: &str = "GET, POST, PUT, PATCH, DELETE, OPTIONS";
const DEFAULT_HEADERS: &str = "Authorization, Content-Type, Accept, X-Requested-With";

#[derive(Debug, Clone)]
pub struct CorsMiddleware {
    allow_origin: String,
    allow_methods: String,
    allow_headers: String,
    allow_credentials: bool,
    max_age_seconds: u32,
}

impl CorsMiddleware {
    pub fn new(origin: impl Into<String>) -> Self {
        Self {
            allow_origin: origin.into(),
            allow_methods: DEFAULT_METHODS.to_string(),
            allow_headers: DEFAULT_HEADERS.to_string(),
            allow_credentials: false,
            max_age_seconds: 86_400,
        }
    }

    pub fn allow_credentials(mut self, value: bool) -> Self {
        self.allow_credentials = value;
        self
    }
}

impl Default for CorsMiddleware {
    fn default() -> Self {
        Self::new("*")
    }
}

#[async_trait]
impl Middleware for CorsMiddleware {
    async fn handle(&self, context: &mut HttpContext, next: Next<'_>) -> Result<(), PipelineError> {
        let method = context.request().method().to_string();
        let response = context.response_mut();
        let headers = response.headers_mut();
        headers.insert("Access-Control-Allow-Origin", self.allow_origin.as_str());
        headers.insert("Access-Control-Allow-Methods", self.allow_methods.as_str());
        headers.insert("Access-Control-Allow-Headers", self.allow_headers.as_str());

        if self.allow_credentials {
            headers.insert("Access-Control-Allow-Credentials", "true");
        }

        if self.max_age_seconds > 0 {
            let max_age = self.max_age_seconds.to_string();
            headers.insert("Access-Control-Max-Age", &max_age);
        }

        if method.eq_ignore_ascii_case("OPTIONS") {
            response.set_status(204);
            response.set_body(ResponseBody::Empty);
            return Ok(());
        }

        next.run(context).await
    }
}
