use super::bot::TestBot;
use super::step::TestStep;
use super::TestResult;
use async_trait::async_trait;

#[async_trait]
pub trait TestScenario: Send + Sync {
    fn name(&self) -> &'static str;

    async fn setup(&self, _bot: &mut TestBot) -> TestResult {
        Ok(())
    }

    fn steps(&self) -> Vec<Box<dyn TestStep>>;

    async fn teardown(&self, _bot: &mut TestBot) -> TestResult {
        Ok(())
    }
}
