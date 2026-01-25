use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use crate::background::job::{BackgroundJob, JobContext, JobResult};
use crate::background::job_queue::JobQueue;
use crate::di::ServiceProvider;

#[derive(Clone)]
pub struct InMemoryJobQueue {
    jobs: Arc<Mutex<VecDeque<Box<dyn BackgroundJob>>>>,
    services: Arc<ServiceProvider>,
}

impl InMemoryJobQueue {
    pub fn new(services: Arc<ServiceProvider>) -> Self {
        Self {
            jobs: Arc::new(Mutex::new(VecDeque::new())),
            services,
        }
    }

    pub fn run_next(&self) -> Option<JobResult> {
        let job = self.jobs.lock().expect("jobs lock").pop_front();
        job.map(|job| job.execute(JobContext::new(self.services.clone())))
    }

    pub fn run_all(&self) -> Vec<JobResult> {
        let mut results = Vec::new();
        while let Some(result) = self.run_next() {
            results.push(result);
        }
        results
    }
}

impl JobQueue for InMemoryJobQueue {
    fn enqueue(&self, job: Box<dyn BackgroundJob>) {
        self.jobs.lock().expect("jobs lock").push_back(job);
    }
}
