use serde::Serialize;
use std::fmt;
use std::future::Future;
use std::pin::Pin;

use log::Level;

use super::assert::Asset;
use super::context::TestContext;
use super::http::{HttpClient, HttpResponse};
use super::scenario::TestScenario;
use super::TestResult;

type ScenarioTask =
    Box<dyn for<'a> FnMut(&'a mut TestBot) -> Pin<Box<dyn Future<Output = TestResult> + 'a>>>;

pub struct TestBot {
    pub context: TestContext,
    client: HttpClient,
    scenarios: Vec<ScenarioTask>,
    steps_run: usize,
}

impl TestBot {
    pub async fn connect(base_url: impl Into<String>) -> TestResult<Self> {
        Ok(Self {
            context: TestContext::default(),
            client: HttpClient::new(base_url),
            scenarios: Vec::new(),
            steps_run: 0,
        })
    }

    pub async fn get(&self, path: &str) -> TestResult<HttpResponse> {
        self.client.get(path, None).await
    }

    pub async fn post<T: Serialize>(&self, path: &str, body: &T) -> TestResult<HttpResponse> {
        self.client.post(path, body, None).await
    }

    pub async fn put<T: Serialize>(&self, path: &str, body: &T) -> TestResult<HttpResponse> {
        self.client.put(path, body, None).await
    }

    pub async fn delete(&self, path: &str) -> TestResult<HttpResponse> {
        self.client.delete(path, None).await
    }

    pub async fn get_auth(&self, path: &str) -> TestResult<HttpResponse> {
        let token = self.context.access_token.as_deref();
        self.client.get(path, token).await
    }

    pub async fn post_auth<T: Serialize>(&self, path: &str, body: &T) -> TestResult<HttpResponse> {
        let token = self.context.access_token.as_deref();
        self.client.post(path, body, token).await
    }

    pub async fn put_auth<T: Serialize>(&self, path: &str, body: &T) -> TestResult<HttpResponse> {
        let token = self.context.access_token.as_deref();
        self.client.put(path, body, token).await
    }

    pub async fn delete_auth(&self, path: &str) -> TestResult<HttpResponse> {
        let token = self.context.access_token.as_deref();
        self.client.delete(path, token).await
    }

    pub fn asset(&mut self) -> Asset<'_> {
        Asset::new(self)
    }

    pub fn log(&self, level: Level, msg: impl fmt::Display) {
        log::log!(level, "          {}", msg);
    }

    pub fn log_info(&self, msg: impl fmt::Display) {
        self.log(Level::Info, msg);
    }

    pub fn assert_equals<T: PartialEq + std::fmt::Debug>(
        &mut self,
        actual: T,
        expected: T,
    ) -> bool {
        self.asset().equals("value", actual, expected)
    }

    pub fn assert_equals_named<T: PartialEq + std::fmt::Debug>(
        &mut self,
        label: impl Into<String>,
        actual: T,
        expected: T,
    ) -> bool {
        self.asset().equals(label, actual, expected)
    }

    pub fn add_scenario<S>(&mut self, scenario: S)
    where
        S: TestScenario + 'static,
    {
        let mut scenario = Some(scenario);
        let task: ScenarioTask = Box::new(move |bot| {
            let scenario = scenario.take().expect("scenario already consumed");
            Box::pin(bot.run_scenario(scenario))
        });
        self.scenarios.push(task);
    }

    pub async fn run(&mut self) -> TestResult {
        let mut scenarios = std::mem::take(&mut self.scenarios);
        for task in scenarios.iter_mut() {
            task(self).await?;
        }

        self.print_result();
        Ok(())
    }

    pub async fn run_scenario<S>(&mut self, scenario: S) -> TestResult
    where
        S: TestScenario,
    {
        log::info!("🤖 Start running scenario: {}", scenario.name());

        scenario.setup(self).await?;
        let steps = scenario.steps();
        self.steps_run += steps.len();
        for step in steps {
            log::info!("  → Step: {} ⇢ {}", step.name(), step.endpoint());

            if let Err(err) = step.run(self).await {
                log::error!(
                    "    ✗ Step: {} ⇢ failed at '{}' ⇢ {}",
                    step.name(),
                    step.endpoint(),
                    err
                );

                self.context.record_assertion_failure(format!(
                    "Step '{}' failed at '{}' ⇢ {}",
                    step.name(),
                    step.endpoint(),
                    err
                ));
                continue;
            }
            log::info!("    ✔     {} ⇢ OK", step.name());
        }
        scenario.teardown(self).await?;
        log::info!("  Finished scenario: {}", scenario.name());
        log::info!("");

        Ok(())
    }

    pub async fn run_scenarios<S>(&mut self, scenarios: Vec<S>) -> TestResult
    where
        S: TestScenario,
    {
        for scenario in scenarios {
            self.run_scenario(scenario).await?;
        }

        self.print_result();
        Ok(())
    }

    fn print_result(&mut self) {
        let failures = self.context.take_assertion_failures();
        let steps = self.steps_run;
        self.steps_run = 0;
        if failures.is_empty() {
            log::info!("✅  All {} steps passed", steps);
            return;
        }

        let count = failures.len();

        log::error!(
            "🔥  {} assertion failure(s) ({} steps processed)",
            count,
            steps
        );
        for (idx, failure) in failures.iter().enumerate() {
            log::error!("  💥 {}. {}", idx + 1, failure);
        }
    }
}
