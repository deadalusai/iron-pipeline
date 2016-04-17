
use iron::prelude::*;
use iron::middleware::{ Handler };

use ::{ Pipeline, PipelineProcessor, PipelineNext };

// Container for a pipeline processor function which must directly handle the request.
//
// # Examples
//
// ```
// pipeline.and(Process(|req| ...))
// ```
pub struct Process<F>(pub F)
    where F: Fn(&mut Request) -> IronResult<Response>,
          F: Send + Sync;

impl <F> PipelineProcessor for Process<F>
    where F: Fn(&mut Request) -> IronResult<Response>,
          F: Send + Sync
{
    fn process(&self, req: &mut Request, _: PipelineNext) -> IronResult<Response> {
        (self.0)(req)
    }
}


// Container for a pipeline processor function which may optionally invoke the
// next handler in the pipeline.
//
// # Examples
//
// ```
// pipeline.add(ProcessNext(|req, next| ...))
// ```
pub struct ProcessNext<F>(pub F)
    where F: Fn(&mut Request, PipelineNext) -> IronResult<Response>,
          F: Send + Sync;

impl <F> PipelineProcessor for ProcessNext<F>
    where F: Fn(&mut Request, PipelineNext) -> IronResult<Response>,
          F: Send + Sync
{
    fn process(&self, req: &mut Request, next: PipelineNext) -> IronResult<Response> {
        (self.0)(req, next)
    }
}


// Processor which optionally delegates processing to a sub pipeline
// based on a request predicate function
pub struct Fork<P>(Pipeline, P);

impl <P> Fork<P>
    where P: Fn(&mut Request) -> bool,
          P: Send + Sync
{
    // Construct a new pipeline fork.
    // The `predicate` is executed on every request and determines whether to delegate to the sub pipeline.
    // The `pipeline_builder` is used to construct the sub pipeline, and is executed immediately.
    pub fn when<B>(predicate: P, pipeline_builder: B) -> Fork<P>
        where B: FnOnce(&mut Pipeline)
    {
        let mut sub_pipeline = Pipeline::new();
        (pipeline_builder)(&mut sub_pipeline);
        Fork(sub_pipeline, predicate)
    }
}

impl <P> PipelineProcessor for Fork<P>
    where P: Fn(&mut Request) -> bool,
          P: Send + Sync
{
    fn process(&self, req: &mut Request, next: PipelineNext) -> IronResult<Response> {
        let Fork(ref sub_pipeline, ref predicate) = *self;
        if (predicate)(req) {
            sub_pipeline.handle(req)
        }
        else {
            next.process(req)
        }
    }
}
