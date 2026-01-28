use std::sync::{Arc, Mutex};

use nimble_web::background::hosted_service::{
    HostedService, HostedServiceContext, HostedServiceHost,
};
use nimble_web::background::in_memory_queue::InMemoryJobQueue;
use nimble_web::background::job::{BackgroundJob, JobContext, JobResult};
use nimble_web::background::job_queue::JobQueue;
use nimble_web::background::runner::JobQueueRunner;
use nimble_web::controller::controller::Controller;

use nimble_web::di::ServiceContainer;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::http::context::HttpContext;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::into_response::ResponseValue;
use nimble_web::testkit::app::TestApp;
use nimble_web::testkit::request::HttpRequestBuilder;
use std::cell::RefCell;
use std::collections::VecDeque;

#[test]
fn hosted_service_lifecycle_calls_start_and_stop() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let service = TestHostedService {
        calls: calls.clone(),
    };

    let mut host = HostedServiceHost::new();
    host.add(service);

    let services = ServiceContainer::new().build();
    host.start(HostedServiceContext::new(services));
    host.stop();

    let snapshot = calls.lock().expect("calls lock").clone();
    assert_eq!(snapshot, vec!["start", "stop"]);
}

#[test]
fn job_queue_executes_jobs() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let queue = TestJobQueue::new();

    queue.enqueue(Box::new(TestJob::new("job-1", calls.clone())));
    queue.run();

    let snapshot = calls.lock().expect("calls lock").clone();
    assert_eq!(snapshot, vec!["job-1"]);
}

#[test]
fn jobs_run_after_request_completes() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let queue = Arc::new(TestJobQueue::new());
    set_test_calls(calls.clone());
    set_test_queue(queue.clone());

    let response = TestApp::new()
        .add_controller::<EnqueueController>()
        .run(HttpRequestBuilder::post("/enqueue").build());

    assert_eq!(response.status(), 200);

    let snapshot = calls.lock().expect("calls lock").clone();
    assert_eq!(snapshot, vec!["handler"]);

    queue.run();

    let snapshot = calls.lock().expect("calls lock").clone();
    assert_eq!(snapshot, vec!["handler", "job"]);
}

#[test]
fn job_queue_executes_in_fifo_order() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let queue = TestJobQueue::new();

    queue.enqueue(Box::new(TestJob::new("first", calls.clone())));
    queue.enqueue(Box::new(TestJob::new("second", calls.clone())));
    queue.enqueue(Box::new(TestJob::new("third", calls.clone())));
    queue.run();

    let snapshot = calls.lock().expect("calls lock").clone();
    assert_eq!(snapshot, vec!["first", "second", "third"]);
}

#[test]
fn job_queue_runs_without_threads() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let queue = TestJobQueue::new();

    queue.enqueue(Box::new(TestJob::new("job", calls.clone())));

    let snapshot = calls.lock().expect("calls lock").clone();
    assert!(snapshot.is_empty());

    queue.run();

    let snapshot = calls.lock().expect("calls lock").clone();
    assert_eq!(snapshot, vec!["job"]);
}

#[test]
fn job_context_exposes_services_and_failure_variant() {
    let services = Arc::new(ServiceContainer::new().build());
    let ctx = JobContext::new(services.clone());
    assert!(ctx.services().resolve::<u8>().is_none());

    let failure = JobResult::Failure("failed".to_string());
    match failure {
        JobResult::Failure(message) => assert_eq!(message, "failed"),
        JobResult::Success => panic!("expected failure"),
    }
}

#[test]
fn in_memory_job_queue_runs_jobs_and_returns_results() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let services = Arc::new(ServiceContainer::new().build());
    let queue = InMemoryJobQueue::new(services);

    queue.enqueue(Box::new(TestJob::new("first", calls.clone())));
    queue.enqueue(Box::new(TestJob::new("second", calls.clone())));

    let result = queue.run_next();
    assert!(matches!(result, Some(JobResult::Success)));

    let results = queue.run_all();
    assert_eq!(results.len(), 1);

    let snapshot = calls.lock().expect("calls lock").clone();
    assert_eq!(snapshot, vec!["first", "second"]);
}

#[test]
fn job_queue_runner_respects_accepting_state() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let services = Arc::new(ServiceContainer::new().build());
    let queue = Arc::new(InMemoryJobQueue::new(services));
    let runner = JobQueueRunner::new(queue.clone());

    runner.stop();
    runner.enqueue(Box::new(TestJob::new("ignored", calls.clone())));
    let results = runner.run_pending_jobs();
    assert!(results.is_empty());

    runner.start(HostedServiceContext::new(ServiceContainer::new().build()));
    runner.enqueue(Box::new(TestJob::new("accepted", calls.clone())));
    let results = runner.run_pending_jobs();
    assert_eq!(results.len(), 1);

    let snapshot = calls.lock().expect("calls lock").clone();
    assert_eq!(snapshot, vec!["accepted"]);
}

struct TestHostedService {
    calls: Arc<Mutex<Vec<&'static str>>>,
}

impl HostedService for TestHostedService {
    fn start(&self, _ctx: HostedServiceContext) {
        self.calls.lock().expect("calls lock").push("start");
    }

    fn stop(&self) {
        self.calls.lock().expect("calls lock").push("stop");
    }
}

struct TestJob {
    label: &'static str,
    calls: Arc<Mutex<Vec<&'static str>>>,
}

impl TestJob {
    fn new(label: &'static str, calls: Arc<Mutex<Vec<&'static str>>>) -> Self {
        Self { label, calls }
    }
}

impl BackgroundJob for TestJob {
    fn execute(&self, _ctx: JobContext) -> JobResult {
        self.calls.lock().expect("calls lock").push(self.label);
        JobResult::Success
    }
}

struct EnqueueController;

use nimble_web::endpoint::route::EndpointRoute;

impl Controller for EnqueueController {
    fn routes() -> Vec<EndpointRoute> {
        vec![EndpointRoute::post("/enqueue", EnqueueHandler).build()]
    }
}

use async_trait::async_trait;

struct EnqueueHandler;

#[async_trait]
impl HttpHandler for EnqueueHandler {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        if let Some(calls) = test_calls() {
            calls.lock().expect("calls lock").push("handler");
        }
        if let Some(queue) = test_queue() {
            let job = TestJob::new("job", test_calls().expect("calls set"));
            queue.enqueue(Box::new(job));
        }
        Ok(ResponseValue::new("queued"))
    }
}

thread_local! {
    static TEST_CALLS: RefCell<Option<Arc<Mutex<Vec<&'static str>>>>> = const { RefCell::new(None) };
    static TEST_QUEUE: RefCell<Option<Arc<TestJobQueue>>> = const { RefCell::new(None) };
}

fn set_test_calls(calls: Arc<Mutex<Vec<&'static str>>>) {
    TEST_CALLS.with(|cell| {
        *cell.borrow_mut() = Some(calls);
    });
}

fn set_test_queue(queue: Arc<TestJobQueue>) {
    TEST_QUEUE.with(|cell| {
        *cell.borrow_mut() = Some(queue);
    });
}

fn test_calls() -> Option<Arc<Mutex<Vec<&'static str>>>> {
    TEST_CALLS.with(|cell| cell.borrow().clone())
}

fn test_queue() -> Option<Arc<TestJobQueue>> {
    TEST_QUEUE.with(|cell| cell.borrow().clone())
}

struct TestJobQueue {
    jobs: Mutex<VecDeque<Box<dyn BackgroundJob>>>,
    services: Arc<nimble_web::di::ServiceProvider>,
}

impl TestJobQueue {
    fn new() -> Self {
        let services = ServiceContainer::new().build();
        Self {
            jobs: Mutex::new(VecDeque::new()),
            services: Arc::new(services),
        }
    }

    fn run(&self) {
        let services = self.services.clone();
        loop {
            let job = {
                let mut guard = self.jobs.lock().expect("jobs lock");
                guard.pop_front()
            };
            let Some(job) = job else {
                break;
            };
            let _ = job.execute(JobContext::new(services.clone()));
        }
    }
}

impl JobQueue for TestJobQueue {
    fn enqueue(&self, job: Box<dyn BackgroundJob>) {
        self.jobs.lock().expect("jobs lock").push_back(job);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
