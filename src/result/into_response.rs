use std::any::Any;

use crate::http::context::HttpContext;
use crate::http::response::HttpResponse;

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
        fn respond<T>(value: Box<dyn Any + Send>, ctx: &mut HttpContext)
        where
            T: IntoResponse + Send + 'static,
        {
            let value = *value.downcast::<T>().expect("response value type mismatch");
            value.into_response(ctx);
        }

        Self {
            value: Box::new(value),
            responder: respond::<T>,
        }
    }

    pub fn apply(self, context: &mut HttpContext) {
        (self.responder)(self.value, context);
    }
}

impl IntoResponse for HttpResponse {
    fn into_response(self, context: &mut HttpContext) {
        *context.response_mut() = self;
    }
}
