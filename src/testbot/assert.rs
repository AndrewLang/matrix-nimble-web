use std::fmt::Debug;

use super::bot::TestBot;
use super::http::HttpResponse;
use super::{TestError, TestResult};

pub trait AssertResponse {
    fn assert_status(&self, expected: u16) -> TestResult<&Self>;
}

impl AssertResponse for HttpResponse {
    fn assert_status(&self, expected: u16) -> TestResult<&Self> {
        if self.status != expected {
            let body = self.text();
            let detail = if body.is_empty() {
                format!("Expected status {}, got {}", expected, self.status)
            } else {
                format!(
                    "Expected status {}, got {}: {}",
                    expected, self.status, body
                )
            };
            return Err(TestError::msg(detail));
        }

        Ok(self)
    }
}

pub struct Asset<'a> {
    bot: &'a mut TestBot,
}

impl<'a> Asset<'a> {
    pub fn new(bot: &'a mut TestBot) -> Self {
        Self { bot }
    }

    pub fn equals<T: PartialEq + Debug>(
        &mut self,
        label: impl Into<String>,
        actual: T,
        expected: T,
    ) -> bool {
        let label = label.into();
        if actual != expected {
            let message = format!(
                "Equals failed for [{}]: expected {expected:?}, got {actual:?}",
                label
            );
            self.bot.context.record_assertion_failure(message);
            false
        } else {
            true
        }
    }
}
