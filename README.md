# iron-pipeline

Simple pipelined request handler for the Iron framework. Inspired by similar APIs used in
the new ASP.NET Core 1.0 framework, which in turn is inspired by Microsoft's OWIN implementation,
node.js and other web frameworks.

# About

Under `iron-pipeline`, every request is sent through a daisy chain of _middlewares_, each of which may
optionally:

1. Create and return a response
2. Modify the request
3. Delegate to the next middleware in the pipeline 
4. Modify the response created by another middleware

Unlike `Chain`, middleware is always executed in the order in which it was registered.

# The `PipelineMiddleware` trait

The `PipelineMiddleware` trait is implemented for any middleware you want to run in a pipeline. The trait is
nearly identical to `iron::middleware::Handler` trait, but for the addition of the `next` parameter.

For example, a simple HTTPS redirect middleware:

```rust
struct HttpsRedirect;
impl PipelineMiddleware for HttpsRedirect {
    fn handle(&self, req: &mut Request, next: PipelineNext) -> IronResult<Response> {
        if req.url.scheme != "https" {
            // Redirect non-https requests to the https version of this endpoint
            let url = make_https_url(&req.url);
            return Ok(Response::with((status::PermanentRedirect, Location(url))));
        }
        // Allow all other requests to continue
        next.process(req)
    }
}

```

# Examples

A simple pipeline:

```rust
use iron::prelude::*;
use iron::status;
use iron_pipeline::prelude::*;

fn main() {
    let mut pipeline = Pipeline::new();
    
    // "Middleware handler" example
    pipeline.add(HandleNext(|req, next| {
        log_request(req);
        let res = next.process(req);
        log_response(&res);
        res
    }));
    
    // "Handler" example
    pipeline.add(Handle(|req| {
        Ok(Response::with((
            status::Ok,
            "Hello from iron-pipeline"
        )))
    }));

    Iron::new(pipeline); // etc...
}
```

Forking a pipeline based on request path or a predicate:

```rust
let api_v1_router = ...;
let api_v2_router = ...;

let mut pipeline = Pipeline::new();

// This middleware runs on all requests
pipeline.add(HttpsRedirect);

// Fork on path prefix example
pipeline.add(Fork::when_path("/api/v2", |v2| {
    // This middleware runs only on requests against /api/v2/*
    v2.add(AuthenticationMiddleware);
    v2.add(api_v2_router);
}));

// Fork on predicate example
pipeline.add(Fork::when(|req| request_has_header(req, "X-ApiVersion", "2009-01-01"), |v1| {
    // This middleware runs only on requests with the correct X-ApiVersion header
    v1.add(OldAuthenticationMiddleware);
    v1.add(api_v1_router);
}));

// "Terminal" handler - returns a 404 to all requests which have not yet been handled
pipeline.add(Handle(|req| {
    Ok(Response::with(status::NotFound))
}));
```

# Usage

This crate may be used by adding `iron-pipeline` to the dependencies in your project's `Cargo.toml`:

```toml
[dependencies]
iron-pipeline = { git = "https://github.com/deadalusai/iron-pipeline" }
```

and the following to your crate root:

```rust
extern crate iron_pipeline;
```