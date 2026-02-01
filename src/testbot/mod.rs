pub mod assert;
pub mod bot;
pub mod context;
pub mod error;
pub mod http;
pub mod combo;
pub mod scenario;
pub mod step;

pub use assert::{AssertResponse, Asset};
pub use bot::TestBot;
pub use context::TestContext;
pub use error::TestError;
pub use http::{HttpClient, HttpResponse};
pub use combo::ComboStep;
pub use scenario::TestScenario;
pub use step::TestStep;

pub type TestResult<T = ()> = Result<T, TestError>;
