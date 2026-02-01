use async_trait::async_trait;

use super::bot::TestBot;
use super::step::TestStep;
use super::TestResult;

pub struct ComboStep {
    name: &'static str,
    endpoint: &'static str,
    steps: Vec<Box<dyn TestStep>>,
}

impl ComboStep {
    pub fn new(name: &'static str, endpoint: &'static str, steps: Vec<Box<dyn TestStep>>) -> Self {
        Self {
            name,
            endpoint,
            steps,
        }
    }
}

#[async_trait(?Send)]
impl TestStep for ComboStep {
    fn name(&self) -> &'static str {
        self.name
    }

    fn endpoint(&self) -> &'static str {
        self.endpoint
    }

    async fn run(&self, bot: &mut TestBot) -> TestResult {
        bot.log_info(format!(
            "⤷ '{}' running {} nested step(s)",
            self.name,
            self.steps.len()
        ));

        for step in &self.steps {
            bot.log_info(format!(
                "⇢ '{}' > step ⇢ '{}' ⇢ {}",
                self.name,
                step.name(),
                step.endpoint()
            ));
            step.run(bot).await?;
        }

        Ok(())
    }
}
