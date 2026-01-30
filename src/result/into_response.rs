use serde::Serialize;
use std::any::Any;

use crate::http::context::HttpContext;
use crate::http::response::HttpResponse;
use crate::result::Json;

pub trait IntoResponse {
    fn into_response(self, context: &mut HttpContext);
}

#[doc(hidden)]
pub struct ResponseValue {
    value: Box<dyn Any + Send>,
    responder: fn(Box<dyn Any + Send>, &mut HttpContext),
}

impl ResponseValue {
    pub fn new<T>(value: T) -> Self
    where
        T: IntoResponse + Send + 'static,
    {
        fn respond<T>(value: Box<dyn Any + Send>, context: &mut HttpContext)
        where
            T: IntoResponse + Send + 'static,
        {
            let value = *value.downcast::<T>().expect("response value type mismatch");
            value.into_response(context);
        }

        Self {
            value: Box::new(value),
            responder: respond::<T>,
        }
    }

    pub fn apply(self, context: &mut HttpContext) {
        (self.responder)(self.value, context);
    }

    pub fn json<T>(value: T) -> Self
    where
        T: Serialize + Send + 'static,
    {
        Self::new(Json(value))
    }
}

impl IntoResponse for HttpResponse {
    fn into_response(self, context: &mut HttpContext) {
        *context.response_mut() = self;
    }
}
