# iron-pipeline

Simple pipelined request handler for the Iron framework. Inspired by similar APIs used in
the new ASP.NET Core 1.0 framework, which in turn is inspired by Microsoft's OWIN implementation,
node.js and other web frameworks.

# About

Under `iron-pipeline` every request is sent through a daisy chain of _middlewares_, each of which may
optionally:

1. Create and return a response
2. Modify the request
3. Delegate to the next middleware in the pipeline 
4. Modify the response created by another middleware

Unlike `Chain`, middleware is always executed in the exact order in which it was registered. Also unlike
`Chain`, there is only one middleware trait: `PipelineMiddleware`.


# The PipelineMiddleware trait

The `PipelineMiddleware` trait is implemented for any middleware you want to run in a pipeline. The trait is
nearly identical to `iron::middleware::Handler` trait, but for the addition of the `next` parameter.

For example, a simple HTTPS redirect middleware:

```rust
struct HttpsRedirect;

impl PipelineMiddleware for HttpsRedirect {
    fn process(&self, req: &mut Request, next: PipelineNext) -> IronResult<Response> {
        if req.url.scheme != "https" {
            // Redirect non-https requests to the https version of this endpoint
            let url = make_https_url(&req.url);
            return Ok(Response::with((status::PermanentRedirect, Location(url))));
        }
        // Allow all other middleware to process the request
        next.process(req)
    }
}
```

Additionally, `PipelineMiddleware` is automatically implemented for all types which implement `Handler`
so you can easily add other Iron-compatible handlers to your pipeline.

**Note:** Because the `Handler` trait does not understand the concept of "next" middleware, it is 
generally only useful to put such handlers at the _end_ of a pipeline or sub-pipeline.

# Examples
See the [examples directory](examples).


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