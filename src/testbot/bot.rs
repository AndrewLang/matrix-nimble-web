use serde::Serialize;

use super::context::TestContext;
use super::http::{HttpClient, HttpResponse};
use super::scenario::TestScenario;
use super::TestResult;

pub struct TestBot {
    pub context: TestContext,
    client: HttpClient,
}

impl TestBot {
    pub async fn connect(base_url: impl Into<String>) -> TestResult<Self> {
        Ok(Self {
            context: TestContext::default(),
            client: HttpClient::new(base_url),
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

    pub async fn run_scenario<S: TestScenario>(&mut self, scenario: S) -> TestResult {
        log::info!("[TestBot] Start run scenario: {}", scenario.name());

        scenario.setup(self).await?;
        for step in scenario.steps() {
            log::info!("  → Step: {} -> {}", step.name(), step.endpoint());
            step.run(self).await?;
            log::info!("    ✔ ok");
        }
        scenario.teardown(self).await?;

        Ok(())
    }
}
