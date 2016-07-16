use iron::prelude::*;
use iron::middleware::Handler;

use url::Url;

use {Pipeline, PipelineMiddleware, PipelineNext};

fn parse_path(s: &str) -> Option<Vec<String>> {
    let base = Url::parse("http://dummy.com").unwrap();
    base.join(s).ok()
        .and_then(|url| url.path_segments()
                           .map(|split| split.map(|s| s.to_string())
                                             .collect()))
}

/// Middleware which optionally delegates to a sub pipeline
/// based on some predicate applied to each request.
pub struct Fork<P>(Pipeline, P);

/// Internal trait used to determine if a request should
/// branch to the sub-pipeline.
#[doc(hidden)]
pub trait ForkPredicate {
    fn matches(&self, req: &Request) -> bool;
}

/// Branch when the request matches the predicate P.
#[doc(hidden)]
pub struct ForkOnFn<P>(P);

impl<P> ForkPredicate for ForkOnFn<P>
    where P: Fn(&Request) -> bool + Send + Sync
{
    fn matches(&self, req: &Request) -> bool {
        let ForkOnFn(ref pred) = *self;
        pred(req)
    }
}

/// Branch when the request URL starts with the given segments.
#[doc(hidden)]
pub struct ForkOnPath(Vec<String>);

impl ForkPredicate for ForkOnPath {
    fn matches(&self, req: &Request) -> bool {
        let ForkOnPath(ref path_segments) = *self;
        slice_starts_with(&req.url.path, path_segments)
    }
}

fn slice_starts_with<A, B>(input: &[A], prefix: &[B]) -> bool
    where A: PartialEq<B>
{
    input.iter().zip(prefix).all(|(a, b)| a == b)
}

impl Fork<()> {
    /// Construct a new pipeline fork.
    /// The `predicate` is executed on every request and determines whether to delegate to the
    /// sub pipeline. The `pipeline_builder` is used to construct the sub pipeline, and is
    /// executed immediately.
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
    /// pipeline.add(Fork::when(|req| req.method == Method::Post, |sub_pipeline| {
    ///     sub_pipeline.add(Handle(|req| {
    ///         Ok(Response::with("Hello from iron-pipeline"))
    ///     }));
    /// }))
    /// # }
    /// ```
    pub fn when<P, B>(predicate: P, pipeline_builder: B) -> Fork<ForkOnFn<P>>
        where B: FnOnce(&mut Pipeline),
              P: Fn(&Request) -> bool + Send + Sync
    {
        let mut sub_pipeline = Pipeline::new();
        pipeline_builder(&mut sub_pipeline);
        Fork(sub_pipeline, ForkOnFn(predicate))
    }

    /// Construct a new pipeline fork.
    /// The `path` is compared against the url path on every request and determines
    /// whether to delegate to the sub pipeline. The `pipeline_builder` is used to
    /// construct the sub pipeline, and is executed immediately.
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
    /// pipeline.add(Fork::when_path("/api/v2", |sub_pipeline| {
    ///     sub_pipeline.add(Handle(|req| {
    ///         Ok(Response::with("Hello from iron-pipeline"))
    ///     }));
    /// }))
    /// # }
    /// ```
    ///
    /// #Panics
    /// Panics when passed an invalid path string. Path should be of the form `/hello/world`.
    pub fn when_path<P, B>(path: P, pipeline_builder: B) -> Fork<ForkOnPath>
        where B: FnOnce(&mut Pipeline),
              P: AsRef<str>
    {
        let segments = parse_path(path.as_ref()).expect("Invalid path");
        let mut sub_pipeline = Pipeline::new();
        pipeline_builder(&mut sub_pipeline);
        Fork(sub_pipeline, ForkOnPath(segments))
    }
}

impl<P> PipelineMiddleware for Fork<P>
    where P: ForkPredicate + Sync + Send
{
    /// Invokes the sub pipeline when the predicate P returns **true** for the request.
    fn process(&self, req: &mut Request, next: PipelineNext) -> IronResult<Response> {
        let Fork(ref sub_pipeline, ref predicate) = *self;
        if predicate.matches(req) {
            sub_pipeline.handle(req)
        }
        else {
            next.process(req)
        }
    }
}
