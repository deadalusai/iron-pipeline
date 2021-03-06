use iron::prelude::*;

use {Middleware, PipelineNext};

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
/// pipeline.add(Handle(|req: &mut Request| {
///     Ok(Response::with((status::Ok, "Hello from iron-pipeline")))
/// }))
/// # }
/// ```
pub struct Handle<F>(pub F)
    where F: Send + Sync + Fn(&mut Request) -> IronResult<Response>;

impl<F> Middleware for Handle<F>
    where F: Send + Sync + Fn(&mut Request) -> IronResult<Response>
{
    fn process(&self, req: &mut Request, _: PipelineNext) -> IronResult<Response> {
        (self.0)(req)
    }
}

/// Container for a pipeline middleware function which may optionally invoke the
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
/// pipeline.add(HandleNext(|req: &mut Request, next: PipelineNext| {
///     log_request(req);
///     let res = next.process(req);
///     log_response(&res);
///     res
/// }))
/// # }
/// ```
pub struct HandleNext<F>(pub F)
    where F: Send + Sync + Fn(&mut Request, PipelineNext) -> IronResult<Response>;

impl<F> Middleware for HandleNext<F>
    where F: Send + Sync + Fn(&mut Request, PipelineNext) -> IronResult<Response>
{
    fn process(&self, req: &mut Request, next: PipelineNext) -> IronResult<Response> {
        (self.0)(req, next)
    }
}
