use crate::http::context::HttpContext;
use crate::http::request::HttpRequest;
use crate::security::auth::User;

pub trait WithAuth {
    fn with_auth(self, user_id: &str) -> Self;
}

impl WithAuth for HttpRequest {
    fn with_auth(mut self, user_id: &str) -> Self {
        let value = format!("Bearer {}", user_id);
        self.headers_mut().insert("authorization", &value);
        self
    }
}

pub fn assert_authenticated(context: &HttpContext) {
    assert!(
        context.get::<User>().is_some(),
        "expected authenticated user in context"
    );
}
