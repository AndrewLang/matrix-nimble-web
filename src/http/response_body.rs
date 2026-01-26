use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::io;

pub trait ResponseBodyStream: Send {
    fn read_chunk(&mut self) -> io::Result<Option<Vec<u8>>>;
}

pub enum ResponseBody {
    Empty,
    Bytes(Vec<u8>),
    Text(String),
    Stream(Box<dyn ResponseBodyStream>),
}

impl Debug for ResponseBody {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ResponseBody::Empty => f.write_str("ResponseBody::Empty"),
            ResponseBody::Bytes(bytes) => {
                f.debug_tuple("ResponseBody::Bytes").field(bytes).finish()
            }
            ResponseBody::Text(text) => f.debug_tuple("ResponseBody::Text").field(text).finish(),
            ResponseBody::Stream(_) => f.write_str("ResponseBody::Stream(..)"),
        }
    }
}

impl Clone for ResponseBody {
    fn clone(&self) -> Self {
        match self {
            ResponseBody::Empty => ResponseBody::Empty,
            ResponseBody::Bytes(bytes) => ResponseBody::Bytes(bytes.clone()),
            ResponseBody::Text(text) => ResponseBody::Text(text.clone()),
            ResponseBody::Stream(_) => ResponseBody::Empty,
        }
    }
}

impl PartialEq for ResponseBody {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ResponseBody::Empty, ResponseBody::Empty) => true,
            (ResponseBody::Bytes(a), ResponseBody::Bytes(b)) => a == b,
            (ResponseBody::Text(a), ResponseBody::Text(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for ResponseBody {}

impl Default for ResponseBody {
    fn default() -> Self {
        ResponseBody::Empty
    }
}
