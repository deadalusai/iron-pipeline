# iron-pipeline

Simple pipelined request handler for the Iron framework. Inspired by similar APIs used in
the new ASP.NET Core 1.0 framework, which in turn is inspired by Microsoft's OWIN implementation,
node.js and other web frameworks.

# Documentation

API documentation can be found [here](http://deadalusai.github.io/iron-pipeline/).


# About

Under `iron-pipeline` every request is sent through a daisy chain of _middlewares_, each of which may
optionally:

1. Create and return a response
2. Modify the request
3. Delegate to the next middleware in the pipeline 
4. Modify the response created by another middleware

Unlike `Chain`, middleware is always executed in the exact order in which it was registered. Also unlike
`Chain`, there is only one middleware trait: `Middleware`.


# The Middleware trait

The `Middleware` trait is implemented for any middleware you want to run in a pipeline.

The trait is nearly identical in behaviour to the `iron::middleware::Handler` trait as it accepts an `&mut Request` and returns
an `IronResult<Response>`. However it also accepts a `PipelineNext` parameter, which allows it to optionally invoke the next
middleware in the pipeline.

For example, a simple HTTPS redirect middleware:

```rust
struct HttpsRedirect;

impl Middleware for HttpsRedirect {
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

Additionally, `Middleware` is automatically implemented for all types which implement `Handler`
so you can easily add other Iron-compatible handlers like `Router` to your pipeline.

**Note:** Because the `Handler` trait cannot invoke the next middleware, it is 
generally only useful to put such handlers at the _end_ of a pipeline or sub-pipeline.

# Examples

See the [examples directory](examples).

You can run the examples with the `cargo run --example` command, e.g.

```bash
$ cargo run --example complex
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