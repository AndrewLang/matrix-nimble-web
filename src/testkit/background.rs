use crate::background::job::{BackgroundJob, JobResult};
use crate::background::job_queue::JobQueue;
use crate::testkit::app::TestApp;

pub trait BackgroundTestkit {
    fn enqueue<J: BackgroundJob + 'static>(&mut self, job: J);
    fn run_background_jobs(&mut self) -> Vec<JobResult>;
}

impl BackgroundTestkit for TestApp {
    fn enqueue<J: BackgroundJob + 'static>(&mut self, job: J) {
        let queue = self.ensure_background_queue();
        queue.enqueue(Box::new(job));
    }

    fn run_background_jobs(&mut self) -> Vec<JobResult> {
        let queue = self.ensure_background_queue();
        queue.run_all()
    }
}
