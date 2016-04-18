use iron::prelude::*;

use ::{ PipelineMiddleware, PipelineNext };

/// Container for a middleware function which must directly handle the request.
///
/// # Examples
///
/// ```rustc
/// pipeline.add(Process(|req: &mut Request| {
///     Ok(Response::with((status::Ok, "Hello from iron-pipeline")))
/// }))
/// ```
pub struct Process<F>(pub F)
    where F: Fn(&mut Request) -> IronResult<Response>,
          F: Send + Sync;

impl <F> PipelineMiddleware for Process<F>
    where F: Fn(&mut Request) -> IronResult<Response>,
          F: Send + Sync
{
    fn process(&self, req: &mut Request, _: PipelineNext) -> IronResult<Response> {
        (self.0)(req)
    }
}

/// Container for a pipeline processor function which may optionally invoke the
/// next middleware in the pipeline via `next.process(req)`, or create a response itself.
///
/// # Examples
///   
/// ```rustc
/// pipeline.add(ProcessNext(|req: &mut Request, next: PipelineNext| {
///     log_request(req);
///     let response = next.process(req);
///     log_response(&response);
///     response
/// }))
/// ```
pub struct ProcessNext<F>(pub F)
    where F: Fn(&mut Request, PipelineNext) -> IronResult<Response>,
          F: Send + Sync;

impl <F> PipelineMiddleware for ProcessNext<F>
    where F: Fn(&mut Request, PipelineNext) -> IronResult<Response>,
          F: Send + Sync
{
    fn process(&self, req: &mut Request, next: PipelineNext) -> IronResult<Response> {
        (self.0)(req, next)
    }
}
