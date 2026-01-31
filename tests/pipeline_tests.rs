use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use nimble_web::config::ConfigBuilder;
use nimble_web::di::ServiceContainer;
use nimble_web::http::context::HttpContext;
use nimble_web::http::request::HttpRequest;
use nimble_web::pipeline::middleware::Middleware;
use nimble_web::pipeline::next::Next;
use nimble_web::pipeline::pipeline::Pipeline;
use nimble_web::pipeline::pipeline::PipelineError;

#[derive(Clone)]
struct Trace {
    steps: Arc<Mutex<Vec<&'static str>>>,
}

impl Trace {
    fn new() -> Self {
        Self {
            steps: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn push(&self, label: &'static str) {
        self.steps.lock().expect("trace lock").push(label);
    }

    fn snapshot(&self) -> Vec<&'static str> {
        self.steps.lock().expect("trace lock").clone()
    }
}

struct OrderMiddleware {
    label: &'static str,
    trace: Trace,
}

#[async_trait]
impl Middleware for OrderMiddleware {
    async fn handle(&self, context: &mut HttpContext, next: Next<'_>) -> Result<(), PipelineError> {
        let _ = context;
        self.trace.push(self.label);
        next.run(context).await
    }
}

struct StopMiddleware {
    label: &'static str,
    trace: Trace,
}

#[async_trait]
impl Middleware for StopMiddleware {
    async fn handle(
        &self,
        context: &mut HttpContext,
        _next: Next<'_>,
    ) -> Result<(), PipelineError> {
        let _ = context;
        self.trace.push(self.label);
        Ok(())
    }
}

struct StatusMiddleware {
    status: u16,
}

#[async_trait]
impl Middleware for StatusMiddleware {
    async fn handle(
        &self,
        context: &mut HttpContext,
        _next: Next<'_>,
    ) -> Result<(), PipelineError> {
        context.response_mut().set_status(self.status);
        Ok(())
    }
}

struct ItemWriter {
    value: u64,
}

#[async_trait]
impl Middleware for ItemWriter {
    async fn handle(&self, context: &mut HttpContext, next: Next<'_>) -> Result<(), PipelineError> {
        context.insert::<u64>(self.value);
        next.run(context).await
    }
}

struct ItemReader {
    expected: u64,
    trace: Trace,
}

#[async_trait]
impl Middleware for ItemReader {
    async fn handle(&self, context: &mut HttpContext, next: Next<'_>) -> Result<(), PipelineError> {
        let value = context.get::<u64>().copied().unwrap_or_default();
        if value == self.expected {
            self.trace.push("seen");
        }
        next.run(context).await
    }
}

struct ErrorMiddleware;

#[async_trait]
impl Middleware for ErrorMiddleware {
    async fn handle(&self, _ctx: &mut HttpContext, _next: Next<'_>) -> Result<(), PipelineError> {
        Err(PipelineError::message("boom"))
    }
}

fn make_context() -> HttpContext {
    let request = HttpRequest::new("GET", "/pipeline");
    let services = ServiceContainer::new().build();
    let config = ConfigBuilder::new().build();
    HttpContext::new(request, services, config)
}

#[test]
fn middleware_executes_in_order() {
    let trace = Trace::new();
    let mut pipeline = Pipeline::new();
    pipeline.add(OrderMiddleware {
        label: "first",
        trace: trace.clone(),
    });
    pipeline.add(OrderMiddleware {
        label: "second",
        trace: trace.clone(),
    });
    pipeline.add(OrderMiddleware {
        label: "third",
        trace: trace.clone(),
    });

    let mut context = make_context();
    let _ = pipeline.run(&mut context);

    assert_eq!(trace.snapshot(), vec!["first", "second", "third"]);
}

#[test]
fn middleware_requires_next_to_continue() {
    let trace = Trace::new();
    let mut pipeline = Pipeline::new();
    pipeline.add(StopMiddleware {
        label: "only",
        trace: trace.clone(),
    });
    pipeline.add(OrderMiddleware {
        label: "after",
        trace: trace.clone(),
    });

    let mut context = make_context();
    let _ = pipeline.run(&mut context);

    assert_eq!(trace.snapshot(), vec!["only"]);
}

#[test]
fn middleware_can_short_circuit_response() {
    let mut pipeline = Pipeline::new();
    pipeline.add(StatusMiddleware { status: 401 });
    pipeline.add(StatusMiddleware { status: 200 });

    let mut context = make_context();
    let _ = pipeline.run(&mut context);

    assert_eq!(context.response().status(), 401);
}

#[test]
fn middleware_mutates_context_items() {
    let trace = Trace::new();
    let mut pipeline = Pipeline::new();
    pipeline.add(ItemWriter { value: 99 });
    pipeline.add(ItemReader {
        expected: 99,
        trace: trace.clone(),
    });

    let mut context = make_context();
    let _ = pipeline.run(&mut context);

    assert_eq!(trace.snapshot(), vec!["seen"]);
}

#[test]
fn middleware_error_stops_pipeline() {
    let trace = Trace::new();
    let mut pipeline = Pipeline::new();
    pipeline.add(ErrorMiddleware);
    pipeline.add(OrderMiddleware {
        label: "after",
        trace: trace.clone(),
    });

    let mut context = make_context();
    let result = pipeline.run(&mut context);

    assert!(matches!(result, Err(PipelineError::Message(msg)) if msg == "boom"));
    assert_eq!(trace.snapshot(), Vec::<&'static str>::new());
}
