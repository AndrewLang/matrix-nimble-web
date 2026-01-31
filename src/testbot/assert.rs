use super::http::HttpResponse;
use super::{TestError, TestResult};

pub trait AssertResponse {
    fn assert_status(&self, expected: u16) -> TestResult<&Self>;
}

impl AssertResponse for HttpResponse {
    fn assert_status(&self, expected: u16) -> TestResult<&Self> {
        if self.status != expected {
            return Err(TestError::msg(format!(
                "❌ Expected status {}, got {}",
                expected, self.status
            )));
        }

        Ok(self)
    }
}
