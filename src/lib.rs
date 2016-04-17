//! Simple request pipeline handler for the Iron framework. Inspired by similar APIs used in
//! the new ASP.NET Core 1.0 framework, which in turn is inspired by Microsoft's OWIN implementation,
//! node.js and other web frameworks.
//! 
//! # Usage
//! 
//! This crate may be used by adding `iron-pipeline` to the dependencies in your project's `Cargo.toml`
//! 
//! **TODO** Add github URL
//! 
//! ```toml
//! [dependencies]
//! iron-pipeline = "0.1"
//! ```
//! 
//! and the following to your crate root:
//! 
//! ```rust
//! extern crate iron_pipeline;
//! ```

extern crate iron;

mod processors;

pub use processors::{ Process, ProcessNext, Fork };

use std::error;
use std::fmt;

use iron::status;
use iron::prelude::*;
use iron::middleware::{ Handler };

mod prelude {
    pub use ::{ Pipeline };
    pub use processors::*;
}

// Trait which defines a request processor within a pipeline.
// Implementors of this trait must call `next.process(...)` in order to pass
// control to the next processor in the pipeline.
pub trait PipelineProcessor: Send + Sync {
    fn process(&self, req: &mut Request, next: PipelineNext) -> IronResult<Response>;
}

// NOTE: Implement PipelineProcessor for all types which also implement Handler

impl <T> PipelineProcessor for T
    where T: Handler
{
    #[inline]
    fn process(&self, req: &mut Request, _: PipelineNext) -> IronResult<Response> {
        self.handle(req)
    }
}

// Middleware for implementing a simple forward-only pipeline.
// When a request is received each handler is invoked in the order
// in which they were registered.
//
// Each handler may modify the request at will. It may then complete the request
// immediately or invoke to the next handler in the pipeline.
//
// # Examples
//
// ```
// let mut pipeline = Pipeline::new();
// pipeline.add(MyCustomRequestPreprocessor);
// pipeline.add(Process(|req| Ok(Response::with((status::Ok, "Hello, world")))));
// Iron::new(pipeline) // Etc
// ```
pub struct Pipeline {
    handlers: Vec<Box<PipelineProcessor>>
}

// Handle used to invoke the next handler in a pipeline
pub struct PipelineNext<'a>(&'a Pipeline, usize);

impl <'a> PipelineNext<'a> {
    pub fn process(&self, req: &mut Request) -> IronResult<Response> {
        let PipelineNext(pipeline, idx) = *self;
        pipeline.invoke_handler(idx, req)
    }
}

impl Pipeline {

    // Construct a new, empty request pipeline.
    pub fn new() -> Pipeline {
        Pipeline {
            handlers: Vec::new()
        }
    }

    // Append a handler to the end of the pipeline
    pub fn add<P>(&mut self, handler: P)
        where P: PipelineProcessor + 'static
    {
        self.handlers.push(Box::new(handler));
    }

    // Invoke the pipeline handler at the given index. The handler is provided
    // With a PipelineNext callback which will invoke the next handler in the
    // pipeline (at position index + 1).
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

// Errors which may be raised by the Pipeline itself
#[derive(Debug, PartialEq)]
pub enum Error {

    // Raised if there are no further handlers in the pipeline
    // available to process the request
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