use crate::http::response::HttpResponse;
use crate::http::response_body::ResponseBody;

pub trait ResponseAssertions {
    fn assert_status(&self, status: u16);
    fn assert_body(&self, body: &str);
    fn assert_json<T: serde::de::DeserializeOwned>(&self) -> T;
}

impl ResponseAssertions for HttpResponse {
    fn assert_status(&self, status: u16) {
        assert_eq!(self.status(), status, "unexpected status");
    }

    fn assert_body(&self, body: &str) {
        assert_eq!(
            self.body(),
            &ResponseBody::Text(body.to_string()),
            "unexpected body"
        );
    }

    fn assert_json<T: serde::de::DeserializeOwned>(&self) -> T {
        match self.body() {
            ResponseBody::Text(payload) => {
                serde_json::from_str(payload).expect("invalid json response body")
            }
            _ => panic!("expected JSON response body"),
        }
    }
}
