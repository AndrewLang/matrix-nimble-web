use crate::http::context::HttpContext;
use crate::pipeline::pipeline::Pipeline;
use crate::pipeline::pipeline::PipelineError;

pub struct Next<'a> {
    pipeline: &'a Pipeline,
    index: usize,
}

impl<'a> Next<'a> {
    pub(crate) fn new(pipeline: &'a Pipeline, index: usize) -> Self {
        Self { pipeline, index }
    }

    pub async fn run(self, context: &mut HttpContext) -> Result<(), PipelineError> {
        if let Some(current) = self.pipeline.middleware().get(self.index) {
            let next = Next::new(self.pipeline, self.index + 1);
            current.handle(context, next).await
        } else {
            Ok(())
        }
    }
}
