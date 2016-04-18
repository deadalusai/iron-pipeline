//! Middleware for use in a `Pipeline`.
//!
//! # A note on `iron::middleware::Handler`
//!
//! All types which implement `iron::middleware::Handler` can also be used as iron-pipeline middleware.
//! However because the `Handler` trait does not understand the concept of "next" middleware,
//! it is generally only useful to put such handlers at the _end_ of a pipeline.
//!
//! For example, using `iron-router`:
//!
//! ```rustc
//! // Ensure the request is coming over HTTPS
//! pipeline.add(ProcessNext(|req, next| {
//!     if req.url.scheme != "https" {
//!         let new_url = build_https_redirect_url(&req.url);
//!         return Ok(Response::with((status::PermanentRedirect, Location(new_url))));
//!     }
//!     next.process(req)
//! }));
//! 
//! pipeline.add(router!(
//!     get  '/'           => entity::handle_get_all,
//!     get  '/entity/:id' => entity::handle_get_by_id,
//!     post '/entity/:id' => entity::handle_update_by_id,
//!     post '/entity/'    => entity::handle_create
//! ));
//! ```



use iron::prelude::*;
use iron::middleware::{ Handler };

use url::parse_path;

use ::{ Pipeline, PipelineMiddleware, PipelineNext };

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


/// Processor which optionally delegates processing to a sub pipeline
/// based on a predicate function executed against each request.
pub struct Fork<P>(Pipeline, P);

impl <P> Fork<P>
    where P: Fn(&Request) -> bool,
          P: Send + Sync
{
    /// Construct a new pipeline fork.
    /// The `predicate` is executed on every request and determines whether to delegate to the sub pipeline.
    /// The `pipeline_builder` is used to construct the sub pipeline, and is executed immediately.
    ///
    /// For example, a sub pipeline which handles all "Post" requests:
    ///
    /// ```rustc
    /// pipeline.add(Fork::when(|req| req.method == Method::Post, |sub_pipeline| {
    ///     sub_pipeline.add(Middleware);
    ///     sub_pipeline.add(Process(...));
    /// }))
    /// ```
    pub fn when<B>(predicate: P, pipeline_builder: B) -> Fork<P>
        where B: FnOnce(&mut Pipeline)
    {
        let mut sub_pipeline = Pipeline::new();
        (pipeline_builder)(&mut sub_pipeline);
        Fork(sub_pipeline, predicate)
    }
}

impl <P> PipelineMiddleware for Fork<P>
    where P: Fn(&Request) -> bool,
          P: Send + Sync
{
    /// Invokes the sub pipeline when the predicate P returns **true** for the request.
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

/// Fork when the request Url matches a hard-coded path prefix.
struct ForkOnPath(Vec<String>);

impl Fork<ForkOnPath> {
    
    /// Construct a new pipeline fork.
    /// The `path` is compared against the url path on every request and determines whether to delegate to the sub pipeline. 
    /// The `pipeline_builder` is used to construct the sub pipeline, and is executed immediately.
    ///
    /// For example, a sub pipeline which handles all requests to "/api/v2":
    ///
    /// ```rustc
    /// pipeline.add(Fork::when_path("/api/v2", |sub_pipeline| {
    ///     sub_pipeline.add(Middleware);
    ///     sub_pipeline.add(Process(...));
    /// }))
    /// ```
    pub fn when_path<P, B>(path: P, pipeline_builder: B) -> Fork<ForkOnPath>
        where P: AsRef<str>,
              B: FnOnce(&mut Pipeline)
    {
        let (segments, _, _) = parse_path(path.as_ref()).expect("Invalid path");
        let mut sub_pipeline = Pipeline::new();
        (pipeline_builder)(&mut sub_pipeline);
        Fork(sub_pipeline, ForkOnPath(segments))
    }
}

impl PipelineMiddleware for Fork<ForkOnPath> {
    /// Invokes the sub pipeline when the request path starts with the given path segments
    fn process(&self, req: &mut Request, next: PipelineNext) -> IronResult<Response> {
        let Fork(ref sub_pipeline, ForkOnPath(ref path_prefix)) = *self;
        if slice_starts_with(&req.url.path, path_prefix) {
            sub_pipeline.handle(req)
        }
        else {
            next.process(req)
        }
    }
}

fn slice_starts_with<A, B>(input: &[A], prefix: &[B]) -> bool
    where A: PartialEq<B>
{
    input.iter().zip(prefix).all(|(a, b)| a == b)
}