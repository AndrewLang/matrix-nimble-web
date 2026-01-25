use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::background::hosted_service::{HostedService, HostedServiceContext};
use crate::background::in_memory_queue::InMemoryJobQueue;
use crate::background::job::JobResult;
use crate::background::job_queue::JobQueue;

pub struct JobQueueRunner {
    queue: Arc<InMemoryJobQueue>,
    accepting: AtomicBool,
}

impl JobQueueRunner {
    pub fn new(queue: Arc<InMemoryJobQueue>) -> Self {
        Self {
            queue,
            accepting: AtomicBool::new(true),
        }
    }

    pub fn run_pending_jobs(&self) -> Vec<JobResult> {
        self.queue.run_all()
    }
}

impl HostedService for JobQueueRunner {
    fn start(&self, _ctx: HostedServiceContext) {
        self.accepting.store(true, Ordering::SeqCst);
    }

    fn stop(&self) {
        self.accepting.store(false, Ordering::SeqCst);
    }
}

impl JobQueue for JobQueueRunner {
    fn enqueue(&self, job: Box<dyn crate::background::job::BackgroundJob>) {
        if self.accepting.load(Ordering::SeqCst) {
            self.queue.enqueue(job);
        }
    }
}
