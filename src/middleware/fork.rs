use iron::prelude::*;
use iron::middleware::{ Handler };

use url::parse_path;

use ::{ Pipeline, PipelineMiddleware, PipelineNext };

/// Middleware which optionally delegates to a sub pipeline
/// based on some predicate executed against each request.
pub struct Fork<P>(Pipeline, P);

/// Internal trait used to implement the predicate on a fork operation. 
pub trait ForkPredicate {
    fn matches(&self, req: &Request) -> bool;
}

impl <P> PipelineMiddleware for Fork<P>
    where P: Send + Sync + ForkPredicate 
{
    /// Invokes the sub pipeline when the predicate P returns **true** for the request.
    fn process(&self, req: &mut Request, next: PipelineNext) -> IronResult<Response> {
        let Fork(ref sub_pipeline, ref pred) = *self;
        if pred.matches(req) {
            sub_pipeline.handle(req)
        }
        else {
            next.process(req)
        }
    }
}


/// Fork when the request matches a predicate
pub struct ForkOnPred<P>(P);

impl <P> ForkPredicate for ForkOnPred<P>
    where P: Send + Sync + Fn(&Request) -> bool
{
    fn matches(&self, req: &Request) -> bool {
        let ForkOnPred(ref predicate) = *self;
        predicate(req)
    }
}

/// Construct a new pipeline fork.
/// The `predicate` is executed on every request and determines whether to delegate to the sub pipeline.
/// The `pipeline_builder` is used to construct the sub pipeline, and is executed immediately.
///
/// # Examples
/// A sub pipeline which handles all "Post" requests:
///
/// ```rust
/// # extern crate iron;
/// # extern crate iron_pipeline;
/// # use iron::prelude::*;
/// # use iron::method::Method;
/// # use iron_pipeline::prelude::*;
/// # fn main() {
/// # let mut pipeline = Pipeline::new();
/// pipeline.add(fork_when(|req| req.method == Method::Post, |sub_pipeline| {
///     sub_pipeline.add(Handle(|req| {
///         Ok(Response::with("Hello from iron-pipeline"))
///     }));
/// }))
/// # }
/// ```
pub fn fork_when<P, B>(predicate: P, pipeline_builder: B) -> Fork<ForkOnPred<P>>
    where P: Send + Sync + Fn(&Request) -> bool,
          B: FnOnce(&mut Pipeline)
{
    let mut sub_pipeline = Pipeline::new();
    pipeline_builder(&mut sub_pipeline);
    Fork(sub_pipeline, ForkOnPred(predicate))
}

/// Fork when the request Url matches a hard-coded path prefix.
pub struct ForkOnPath(Vec<String>);

impl ForkPredicate for ForkOnPath {
    /// Invokes the sub pipeline when the request path starts with the given path segments
    fn matches(&self, req: &Request) -> bool {
        let ForkOnPath(ref path_prefix) = *self;
        slice_starts_with(&req.url.path, path_prefix)
    }
}

fn slice_starts_with<A, B>(input: &[A], prefix: &[B]) -> bool
    where A: PartialEq<B>
{
    input.iter().zip(prefix).all(|(a, b)| a == b)
}

/// Construct a new pipeline fork.
/// The `path` is compared against the url path on every request and determines whether to delegate to the sub pipeline. 
/// The `pipeline_builder` is used to construct the sub pipeline, and is executed immediately.
///
/// # Examples
/// A sub pipeline which handles all requests to "/api/v2":
///
/// ```rust
/// # extern crate iron;
/// # extern crate iron_pipeline;
/// # use iron::prelude::*;
/// # use iron_pipeline::prelude::*;
/// # fn main() {
/// # let mut pipeline = Pipeline::new();
/// pipeline.add(fork_when_path("/api/v2", |sub_pipeline| {
///     sub_pipeline.add(Handle(|req| {
///         Ok(Response::with("Hello from iron-pipeline"))
///     }));
/// }))
/// # }
/// ```
///
/// #Panics
/// Panics when passed an invalid path string. Path should be of the form `/hello/world`.
pub fn fork_when_path<P, B>(path: P, pipeline_builder: B) -> Fork<ForkOnPath>
    where P: AsRef<str>,
          B: FnOnce(&mut Pipeline)
{
    let (segments, _, _) = parse_path(path.as_ref()).expect("Invalid path");
    let mut sub_pipeline = Pipeline::new();
    pipeline_builder(&mut sub_pipeline);
    Fork(sub_pipeline, ForkOnPath(segments))
}