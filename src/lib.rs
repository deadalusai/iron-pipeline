//! Simple pipelined request handler for the Iron framework. Inspired by similar
//! APIs used in the new ASP.NET Core 1.0 framework, which in turn is inspired by
//! Microsoft's OWIN implementation, node.js and other web frameworks.
//!
//! # About
//!
//! Under `iron-pipeline`, every request is sent through a daisy chain of
//! _middlewares_, each of which may optionally:
//!
//! 1. Create and return a response
//! 2. Modify the request
//! 3. Delegate to the next middleware in the pipeline
//! 4. Modify the response created by another middleware
//!
//! Unlike `Chain`, middleware is always executed in the order in which it was registered.
//!
//! # Examples
//!
//! This example introduces two helper middlewares: `Handle` and `HandleNext`.
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
//!     pipeline.add(HandleNext(|req, next| {
//!         log_request(req);
//!         let res = next.process(req);
//!         log_response(&res);
//!         res
//!     }));
//!
//!     // "Handler" example
//!     pipeline.add(Handle(|req| {
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
//! This crate may be used by adding `iron-pipeline` to the dependencies
//! in your project's `Cargo.toml`:
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

pub mod middleware;

/// Includes the Pipeline type and all middleware types in the `middleware` module.
pub mod prelude {
    pub use Pipeline;
    pub use middleware::fork::Fork;
    pub use middleware::handle::{Handle, HandleNext};
}

use std::error;
use std::fmt;

use iron::status;
use iron::prelude::*;
use iron::middleware::Handler;

/// Trait which defines middleware within a pipeline.
/// Implementors of this trait must call `next.handle(...)` in order to pass
/// control to the next middleware in the pipeline.
pub trait Middleware: Send + Sync {
    fn process(&self, req: &mut Request, next: PipelineNext) -> IronResult<Response>;
}

// NOTE: Implement Middleware for all types which also implement Handler

impl<T> Middleware for T
    where T: Handler
{
    #[inline]
    fn process(&self, req: &mut Request, _: PipelineNext) -> IronResult<Response> {
        self.handle(req)
    }
}

/// Iron middleware for implementing a simple forward-only pipeline.
/// When a request is received each middleware is invoked in the order
/// in which they were registered.
///
/// Each middleware may modify the request at will. It may then complete the request
/// immediately or invoke to the next middleware in the pipeline.
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
/// # use iron_pipeline::{ Middleware, PipelineNext };
/// # struct MyCustomRequestPreprocessor;
/// # impl Middleware for MyCustomRequestPreprocessor {
/// #     fn process(&self, req: &mut Request, next: PipelineNext) -> IronResult<Response> {
/// #         next.process(req)
/// #     }
/// # }
/// let mut pipeline = Pipeline::new();
/// pipeline.add(MyCustomRequestPreprocessor);
/// pipeline.add(Handle(|req| Ok(Response::with((status::Ok, "Hello, world")))));
/// Iron::new(pipeline); // Etc...
/// # }
/// ```
pub struct Pipeline {
    middlewares: Vec<Box<Middleware>>
}

/// Handle used to invoke the next handler in a pipeline
pub struct PipelineNext<'a>(&'a Pipeline, usize);

impl<'a> PipelineNext<'a> {
    pub fn process(&self, req: &mut Request) -> IronResult<Response> {
        let PipelineNext(pipeline, idx) = *self;
        pipeline.invoke_handler(idx, req)
    }
}

impl Pipeline {
    /// Construct a new, empty request pipeline.
    pub fn new() -> Pipeline {
        Pipeline { middlewares: Vec::new() }
    }

    /// Append a middleware to the end of the pipeline
    pub fn add<P>(&mut self, handler: P)
        where P: Middleware + 'static
    {
        self.middlewares.push(Box::new(handler));
    }

    /// Invoke the pipeline handler at the given index. The handler is provided
    /// With a PipelineNext callback which will invoke the next handler in the
    /// pipeline (at position index + 1).
    fn invoke_handler(&self, index: usize, req: &mut Request) -> IronResult<Response> {

        // Locate the next handler and invoke it
        if let Some(middleware) = self.middlewares.get(index) {
            return middleware.process(req, PipelineNext(self, index + 1));
        }

        // No more middlewares? Return an error to the client
        Err(IronError::new(Error::NoHandler, status::InternalServerError))
    }
}

impl Handler for Pipeline {
    /// Invokes the request pipeline
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        self.invoke_handler(0, req)
    }
}

/// Errors which may be raised by the Pipeline itself
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Raised if there are no further middlewares in the pipeline
    /// available to handle the request
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
