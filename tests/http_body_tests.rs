use std::sync::{Arc, Mutex};

use nimble_web::http::headers::HttpHeaders;
use nimble_web::http::request_body::{RequestBody, RequestBodyStream, RequestBodyStreamHandle};
use nimble_web::http::response_body::{ResponseBody, ResponseBodyStream};

struct EmptyStream;

impl RequestBodyStream for EmptyStream {
    fn read_chunk(&mut self) -> std::io::Result<Option<Vec<u8>>> {
        Ok(None)
    }
}

struct ResponseStream;

impl ResponseBodyStream for ResponseStream {
    fn read_chunk(&mut self) -> std::io::Result<Option<Vec<u8>>> {
        Ok(None)
    }
}

#[test]
fn request_body_debug_and_equality() {
    let empty = RequestBody::Empty;
    assert_eq!(format!("{:?}", empty), "Empty");
    assert_eq!(empty, RequestBody::default());

    let bytes = RequestBody::Bytes(vec![1, 2, 3]);
    assert!(format!("{:?}", bytes).contains("Bytes"));
    assert_eq!(bytes, RequestBody::Bytes(vec![1, 2, 3]));

    let text = RequestBody::Text("hello".to_string());
    assert!(format!("{:?}", text).contains("Text"));
    assert_eq!(text, RequestBody::Text("hello".to_string()));

    let stream: RequestBodyStreamHandle = Arc::new(Mutex::new(EmptyStream));
    let same = RequestBody::Stream(stream.clone());
    let other = RequestBody::Stream(Arc::new(Mutex::new(EmptyStream)));

    assert_eq!(same, RequestBody::Stream(stream.clone()));
    assert_ne!(same, other);
    assert!(format!("{:?}", same).contains("Stream"));
}

#[test]
fn response_body_clone_and_equality() {
    let bytes = ResponseBody::Bytes(vec![9, 8, 7]);
    assert!(format!("{:?}", bytes).contains("ResponseBody::Bytes"));
    assert_eq!(bytes, bytes.clone());

    let text = ResponseBody::Text("ok".to_string());
    assert!(format!("{:?}", text).contains("ResponseBody::Text"));
    assert_eq!(text, text.clone());

    let empty = ResponseBody::Empty;
    assert!(format!("{:?}", empty).contains("ResponseBody::Empty"));

    let stream = ResponseBody::Stream(Box::new(ResponseStream));
    assert!(format!("{:?}", stream).contains("ResponseBody::Stream"));
    let cloned = stream.clone();
    assert_eq!(cloned, ResponseBody::Empty);
}

#[test]
fn http_headers_iterates_inserted_values() {
    let mut headers = HttpHeaders::new();
    headers.insert("x-one", "1");
    headers.insert("x-two", "2");

    let mut collected = headers
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect::<Vec<_>>();
    collected.sort();

    assert_eq!(
        collected,
        vec![
            ("x-one".to_string(), "1".to_string()),
            ("x-two".to_string(), "2".to_string())
        ]
    );
    assert_eq!(headers.get("x-one"), Some("1"));
    assert_eq!(headers.get("missing"), None);
}
