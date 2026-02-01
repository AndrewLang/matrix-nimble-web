use super::{TestError, TestResult};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Clone)]
pub struct HttpClient {
    base_url: String,
    client: Client,
}

impl HttpClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: Client::new(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url.trim_end_matches('/'), path)
    }

    fn with_bearer<'a>(
        &self,
        builder: reqwest::RequestBuilder,
        bearer: Option<&'a str>,
    ) -> reqwest::RequestBuilder {
        if let Some(token) = bearer {
            builder.bearer_auth(token)
        } else {
            builder
        }
    }

    async fn execute(
        &self,
        builder: reqwest::RequestBuilder,
        bearer: Option<&str>,
    ) -> TestResult<HttpResponse> {
        let builder = self.with_bearer(builder, bearer);
        let response = builder
            .send()
            .await
            .map_err(|err| TestError::msg(err.to_string()))?;

        let status = response.status().as_u16();
        let body = response
            .bytes()
            .await
            .map_err(|err| TestError::msg(err.to_string()))?;
        let body_vec = body.to_vec();
        let body_text = String::from_utf8_lossy(&body_vec).to_string();

        Ok(HttpResponse {
            status,
            body: body_vec,
            body_text,
        })
    }

    pub async fn get(&self, path: &str, bearer: Option<&str>) -> TestResult<HttpResponse> {
        let url = self.url(path);
        self.execute(self.client.get(url), bearer).await
    }

    pub async fn post<T: Serialize>(
        &self,
        path: &str,
        body: &T,
        bearer: Option<&str>,
    ) -> TestResult<HttpResponse> {
        let url = self.url(path);
        self.execute(self.client.post(url).json(body), bearer).await
    }

    pub async fn put<T: Serialize>(
        &self,
        path: &str,
        body: &T,
        bearer: Option<&str>,
    ) -> TestResult<HttpResponse> {
        let url = self.url(path);
        self.execute(self.client.put(url).json(body), bearer).await
    }

    pub async fn delete(&self, path: &str, bearer: Option<&str>) -> TestResult<HttpResponse> {
        let url = self.url(path);
        self.execute(self.client.delete(url), bearer).await
    }
}

pub struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
    body_text: String,
}

impl HttpResponse {
    pub fn json<T: DeserializeOwned>(&self) -> TestResult<T> {
        serde_json::from_slice(&self.body).map_err(|e| TestError::msg(e.to_string()))
    }

    pub fn text(&self) -> &str {
        &self.body_text
    }
}
