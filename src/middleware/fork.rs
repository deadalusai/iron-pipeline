use iron;
use iron::prelude::*;
use iron::middleware::Handler;

use std::error;
use std::fmt;

use {Pipeline, Middleware, PipelineNext};

// Track errors parsing fork paths
#[derive(Debug, PartialEq)]
enum ParsePathError {
    NoLeadingSlash,
    PathEmpty
}

impl fmt::Display for ParsePathError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", error::Error::description(self))
    }
}

impl error::Error for ParsePathError {
    fn description(&self) -> &'static str {
        match *self {
            ParsePathError::NoLeadingSlash => "Path must start with /",
            ParsePathError::PathEmpty      => "Path cannot be empty"
        }
    }
}

fn parse_path(path: &str) -> Result<Vec<String>, ParsePathError> {
    if !path.starts_with("/") {
        return Err(ParsePathError::NoLeadingSlash);
    }

    let segments: Vec<_> =
        path.trim_left_matches("/").split("/")
            .filter(|s| s.len() > 0)
            .map(|s| s.to_string())
            .collect();

    if segments.len() == 0 {
        return Err(ParsePathError::PathEmpty);
    }

    Ok(segments)
}

/// Middleware which optionally delegates to a sub pipeline
/// based on some predicate applied to each request.
pub struct Fork<P>(Pipeline, P);

/// Internal trait used to determine if a request should
/// branch to the sub-pipeline.
#[doc(hidden)]
pub trait ForkHandler {
    fn should_fork(&self, req: &Request) -> bool;
    fn modify_request(&self, _: &mut Request) {
        // nop
    }
}

/// Branch when the request matches the predicate P.
#[doc(hidden)]
pub struct ForkOnFn<P>(P);

impl<P> ForkHandler for ForkOnFn<P>
    where P: Fn(&Request) -> bool + Send + Sync
{
    fn should_fork(&self, req: &Request) -> bool {
        let ForkOnFn(ref pred) = *self;
        pred(req)
    }
}

/// Branch when the request URL starts with the given segments.
#[doc(hidden)]
pub struct ForkOnPath(Vec<String>);

pub struct OriginalUrl;
impl iron::typemap::Key for OriginalUrl {
    type Value = iron::Url;
}

impl ForkHandler for ForkOnPath {
    fn should_fork(&self, req: &Request) -> bool {
        let ForkOnPath(ref path_segments) = *self;
        slice_starts_with(&req.url.path(), path_segments)
    }

    fn modify_request(&self, req: &mut Request) {
        let ForkOnPath(ref path_segments) = *self;

        // Strip the path prefix from the Url
        let new_path = req.url.path()[path_segments.len()..].join("/");
        let mut new_url = req.url.clone().into_generic_url();
        new_url.set_path(&new_path);

        // Make the original (root) URL accessible
        req.extensions.entry::<OriginalUrl>().or_insert(req.url.clone());

        // Overwrite the request Url
        req.url = iron::Url::from_generic_url(new_url).unwrap();
    }
}

fn slice_starts_with<A, B>(input: &[A], prefix: &[B]) -> bool
    where A: PartialEq<B>
{
    if prefix.len() > input.len() {
        return false;
    }

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
        let segments = parse_path(path.as_ref()).unwrap();
        let mut sub_pipeline = Pipeline::new();
        pipeline_builder(&mut sub_pipeline);
        Fork(sub_pipeline, ForkOnPath(segments))
    }
}

impl<P> Middleware for Fork<P>
    where P: ForkHandler + Sync + Send
{
    /// Invokes the sub pipeline when the predicate P returns **true** for the request.
    fn process(&self, req: &mut Request, next: PipelineNext) -> IronResult<Response> {
        let Fork(ref sub_pipeline, ref handler) = *self;
        if handler.should_fork(req) {
            handler.modify_request(req);
            sub_pipeline.handle(req)
        }
        else {
            next.process(req)
        }
    }
}

#[cfg(test)]
mod tests {

    use super::parse_path;
    use super::ParsePathError;

    #[test]
    fn parse_path_ok() {
        let path = parse_path("/this/is/the/path").unwrap();
        assert_eq!(path, vec!["this", "is", "the", "path"]);
    }

    #[test]
    fn parse_path_strips_empty_segments() {
        let path = parse_path("/hello//world").unwrap();
        assert_eq!(path, vec!["hello", "world"]);
    }

    #[test]
    fn parse_path_require_leading_slash() {
        let path = parse_path("this/is/the/path");
        assert_eq!(path, Err(ParsePathError::NoLeadingSlash));
    }

    #[test]
    fn parse_path_require_non_empty() {
        let path = parse_path("/");
        assert_eq!(path, Err(ParsePathError::PathEmpty));
    }

    use super::slice_starts_with;

    #[test]
    fn slice_starts_with_detects_invalid_prefix() {
        let input  = ['1', '2', '3'];
        let prefix = ['9', '9', '9'];
        assert_eq!(false, slice_starts_with(&input, &prefix));
    }

    #[test]
    fn slice_starts_with_prefix_and_input_same_length() {
        let input  = ['1', '2', '3'];
        let prefix = ['1', '2', '3'];
        assert_eq!(true, slice_starts_with(&input, &prefix));
    }

    #[test]
    fn slice_starts_with_longer_input() {
        let input  = ['1', '2', '3', '4'];
        let prefix = ['1', '2', '3'];
        assert_eq!(true, slice_starts_with(&input, &prefix));
    }

    #[test]
    fn slice_starts_with_longer_prefix() {
        let input  = ['1', '2', '3'];
        let prefix = ['1', '2', '3', '4'];
        assert_eq!(false, slice_starts_with(&input, &prefix));
    }
}
