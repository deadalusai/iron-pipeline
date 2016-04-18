use iron::prelude::*;

use ::{ PipelineMiddleware, PipelineNext };

/// Container for a middleware function which must directly handle the request.
///
/// # Examples
///
/// ```rust
/// # extern crate iron;
/// # extern crate iron_pipeline;
/// # use iron::prelude::*;
/// # use iron::status;
/// # use iron_pipeline::prelude::*;
/// # fn main() {
/// # let mut pipeline = Pipeline::new();
/// pipeline.add(Process(|req: &mut Request| {
///     Ok(Response::with((status::Ok, "Hello from iron-pipeline")))
/// }))
/// # }
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
/// ```rust
/// # extern crate iron;
/// # extern crate iron_pipeline;
/// # use iron::prelude::*;
/// # use iron_pipeline::prelude::*;
/// # use iron_pipeline::{ PipelineNext };
/// # fn log_request(_: &Request) {}
/// # fn log_response(_: &IronResult<Response>) {}
/// # fn main() {
/// # let mut pipeline = Pipeline::new();
/// pipeline.add(ProcessNext(|req: &mut Request, next: PipelineNext| {
///     log_request(req);
///     let res = next.process(req);
///     log_response(&res);
///     res
/// }))
/// # }
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
