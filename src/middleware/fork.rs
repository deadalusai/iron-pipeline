use iron::prelude::*;
use iron::middleware::{ Handler };

use url::parse_path;

use ::{ Pipeline, PipelineMiddleware, PipelineNext };

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