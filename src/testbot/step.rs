use super::bot::TestBot;
use async_trait::async_trait;

use super::TestResult;

#[async_trait(?Send)]
pub trait TestStep {
    fn name(&self) -> &'static str;

    fn endpoint(&self) -> &'static str;

    async fn run(&self, bot: &mut TestBot) -> TestResult;
}
