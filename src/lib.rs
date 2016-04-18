//! Simple pipelined request handler for the Iron framework. Inspired by similar APIs used in
//! the new ASP.NET Core 1.0 framework, which in turn is inspired by Microsoft's OWIN implementation,
//! node.js and other web frameworks.
//! 
//! # About
//! 
//! Under `iron-pipeline`, every request is sent through a daisy chain of _middlewares_, each of which may
//! optionally:
//! 
//! 1. Create and return a response
//! 2. Modify the request
//! 3. Delegate to the next processor in the pipeline 
//! 4. Modify the response created by another processor
//! 
//! Unlike `Chain`, middleware is always executed in the order in which it was registered.
//! 
//! # Examples
//!
//! This example introduces two helper processors: `Process` and `ProcessNext`. 
//! 
//! ```rust
//! # extern crate iron;
//! # extern crate iron_pipeline;
//! use iron::prelude::*;
//! use iron::status;
//! use iron_pipeline::prelude::*;
//! 
//! # fn log_request(_: &Request) {}
//! # fn log_response(_: &IronResult<Response>) {}
//!
//! fn main() {
//!     let mut pipeline = Pipeline::new();
//!     
//!     // "Middleware" example
//!     pipeline.add(ProcessNext(|req, next| {
//!         log_request(req);
//!         let res = next.process(req);
//!         log_response(&res);
//!         res
//!     }));
//!     
//!     // "Handler" example
//!     pipeline.add(Process(|req| {
//!         Ok(Response::with((
//!             status::Ok,
//!             "Hello from iron-pipeline"
//!         )))
//!     }));
//!
//!     Iron::new(pipeline); // etc...
//! }
//! ```
//!
//! # Usage
//! 
//! **TODO** Add github URL
//! 
//! This crate may be used by adding `iron-pipeline` to the dependencies in your project's `Cargo.toml`:
//! 
//! ```toml
//! [dependencies]
//! iron-pipeline = { git = "https://github.com/deadalusai/iron-pipeline" }
//! ```
//! 
//! and the following to your crate root:
//! 
//! ```rust
//! extern crate iron_pipeline;
//! ```

extern crate iron;
extern crate url;

pub mod middleware;

/// Includes the Pipeline type and all middleware types in the `middleware` module.
pub mod prelude {
    pub use ::{ Pipeline };
    pub use middleware::fork::{ Fork };
    pub use middleware::process::{ Process, ProcessNext };
}

use std::error;
use std::fmt;

use iron::status;
use iron::prelude::*;
use iron::middleware::{ Handler };

/// Trait which defines middleware within a pipeline.
/// Implementors of this trait must call `next.process(...)` in order to pass
/// control to the next processor in the pipeline.
pub trait PipelineMiddleware: Send + Sync {
    fn process(&self, req: &mut Request, next: PipelineNext) -> IronResult<Response>;
}

// NOTE: Implement PipelineMiddleware for all types which also implement Handler

impl <T> PipelineMiddleware for T
    where T: Handler
{
    #[inline]
    fn process(&self, req: &mut Request, _: PipelineNext) -> IronResult<Response> {
        self.handle(req)
    }
}

/// Iron middleware for implementing a simple forward-only pipeline.
/// When a request is received each handler is invoked in the order
/// in which they were registered.
///
/// Each handler may modify the request at will. It may then complete the request
/// immediately or invoke to the next handler in the pipeline.
///
/// # Examples
///
/// ```rust
/// # extern crate iron;
/// # extern crate iron_pipeline;
/// # fn main() {
/// # use iron::prelude::*;
/// # use iron::status;
/// # use iron_pipeline::prelude::*;
/// # use iron_pipeline::{ PipelineMiddleware, PipelineNext };
/// # struct MyCustomRequestPreprocessor;
/// # impl PipelineMiddleware for MyCustomRequestPreprocessor {
/// #     fn process(&self, req: &mut Request, next: PipelineNext) -> IronResult<Response> {
/// #         next.process(req)
/// #     }
/// # }
/// let mut pipeline = Pipeline::new();
/// pipeline.add(MyCustomRequestPreprocessor);
/// pipeline.add(Process(|req| Ok(Response::with((status::Ok, "Hello, world")))));
/// Iron::new(pipeline); // Etc...
/// # }
/// ```
pub struct Pipeline {
    handlers: Vec<Box<PipelineMiddleware>>
}

/// Handle used to invoke the next handler in a pipeline
pub struct PipelineNext<'a>(&'a Pipeline, usize);

impl <'a> PipelineNext<'a> {
    pub fn process(&self, req: &mut Request) -> IronResult<Response> {
        let PipelineNext(pipeline, idx) = *self;
        pipeline.invoke_handler(idx, req)
    }
}

impl Pipeline {

    /// Construct a new, empty request pipeline.
    pub fn new() -> Pipeline {
        Pipeline {
            handlers: Vec::new()
        }
    }

    /// Append a middleware to the end of the pipeline
    pub fn add<P>(&mut self, handler: P)
        where P: PipelineMiddleware + 'static
    {
        self.handlers.push(Box::new(handler));
    }

    /// Invoke the pipeline handler at the given index. The handler is provided
    /// With a PipelineNext callback which will invoke the next handler in the
    /// pipeline (at position index + 1).
    fn invoke_handler(&self, index: usize, req: &mut Request) -> IronResult<Response> {

        // Locate the next handler and invoke it
        if let Some(handler) = self.handlers.get(index) {
            return handler.process(req, PipelineNext(self, index + 1));
        }

        // No more handlers? Return an error to the client
        Err(IronError::new(Error::NoHandler, status::InternalServerError))
    }
}

impl Handler for Pipeline {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        self.invoke_handler(0, req)
    }
}

/// Errors which may be raised by the Pipeline itself
#[derive(Debug, PartialEq)]
pub enum Error {

    /// Raised if there are no further handlers in the pipeline
    /// available to process the request
    NoHandler
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Pipeline error ({})", error::Error::description(self))
    }
}

impl error::Error for Error {
    fn description(&self) -> &'static str {
        match self {
            &Error::NoHandler => "Missing handler"
        }
    }
}