use crate::background::job::BackgroundJob;

pub trait JobQueue: Send + Sync {
    fn enqueue(&self, job: Box<dyn BackgroundJob>);
}
