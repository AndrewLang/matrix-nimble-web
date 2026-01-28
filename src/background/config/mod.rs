use crate::background::job_queue::JobQueue;
use std::sync::Arc;

pub enum JobQueueConfig {
    None,
    Provided(Arc<dyn JobQueue>),
    InMemory,
}
