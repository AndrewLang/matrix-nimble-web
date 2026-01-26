use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::{Arc, Mutex};

pub trait RequestBodyStream: Send {
    fn read_chunk(&mut self) -> std::io::Result<Option<Vec<u8>>>;
}

pub type RequestBodyStreamHandle = Arc<Mutex<dyn RequestBodyStream>>;

#[derive(Clone)]
pub enum RequestBody {
    Empty,
    Bytes(Vec<u8>),
    Text(String),
    Stream(RequestBodyStreamHandle),
}

impl Debug for RequestBody {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            RequestBody::Empty => f.write_str("Empty"),
            RequestBody::Bytes(bytes) => f.debug_tuple("Bytes").field(bytes).finish(),
            RequestBody::Text(text) => f.debug_tuple("Text").field(text).finish(),
            RequestBody::Stream(_) => f.write_str("Stream(..)"),
        }
    }
}

impl PartialEq for RequestBody {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RequestBody::Empty, RequestBody::Empty) => true,
            (RequestBody::Bytes(left), RequestBody::Bytes(right)) => left == right,
            (RequestBody::Text(left), RequestBody::Text(right)) => left == right,
            (RequestBody::Stream(left), RequestBody::Stream(right)) => Arc::ptr_eq(left, right),
            _ => false,
        }
    }
}

impl Eq for RequestBody {}

impl Default for RequestBody {
    fn default() -> Self {
        RequestBody::Empty
    }
}
