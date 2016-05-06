extern crate iron;
extern crate iron_pipeline;

use iron::prelude::*;
use iron::status;
use iron::middleware::{ Handler };

use iron_pipeline::prelude::*;
use iron_pipeline::{ PipelineMiddleware, PipelineNext };

fn log_request(req: &Request) {
    println!("{} {}", req.method, req.url);
}

fn log_response(res: &Response) {
    match res.status {
        Some(status) => println!("{}", status),
        None         => println!("No response status set")
    }
}

fn main() {
    
    let mut pipeline = Pipeline::new();

    // This middleware runs on all requests
    pipeline.add(HandleNext(|req, next| {
        // Log the request and response
        log_request(req);
        match next.process(req) {
            Ok(res) => {
                log_response(&res);
                Ok(res)
            },
            Err(err) => {
                println!("Error {}", err.error);
                log_response(&err.response);
                Err(err)
            }
        }
    }));
    
    // Example of forking on a predicate 
    pipeline.add(fork_when(|req| request_has_header(req, "X-ApiVersion", b"2009-01-01"), |v1| {
        // This middleware runs only on requests with the correct X-ApiVersion header
        v1.add(WwwAuthenticate { username: "v1", password: "password" });
        v1.add(ApiV1Handler);
    }));

    // Example of forking on path prefix
    pipeline.add(fork_when_path("/api/v2", |v2| {
        // This middleware runs only on requests where the path starts with /api/v2/*
        v2.add(WwwAuthenticate { username: "v2", password: "password" });
        v2.add(ApiV2Handler);
    }));

    // Example inline handler - returns a 404 to all requests which did not 
    // match one of the above forks
    pipeline.add(Handle(|_| {
        Ok(Response::with((status::NotFound, "Not Found")))
    }));
    
    let port = 1337;
    let _listener =
        Iron::new(pipeline)
            .http(("localhost", port))
            .unwrap();
            
    println!("Listening on port {}", port);
}

/// Utility function for checking a custom header value
fn request_has_header(req: &Request, header_name: &str, header_value: &[u8]) -> bool {
    if let Some(values) = req.headers.get_raw(header_name) {
        values.iter().all(|v| v == &header_value)
    }
    else {
        false
    }
}

/// Simple PipelineMiddleware which challenges all 
/// requests for the configured username and password
struct WwwAuthenticate {
    username: &'static str,
    password: &'static str
}

impl PipelineMiddleware for WwwAuthenticate {
    fn process(&self, req: &mut Request, next: PipelineNext) -> IronResult<Response> {
        use iron::headers::{ Authorization, Basic };
        
        let user_authorized =
            match req.headers.get::<Authorization<Basic>>() {
                Some(&Authorization(ref basic)) => {
                    // Valid credentials?
                    let username_ok = basic.username == self.username;
                    let password_ok = basic.password.as_ref().map(|p| p == self.password).unwrap_or(false);
                    username_ok && password_ok
                },
                _ => false,
            };
        
        if !user_authorized {
            // Challenge the user to authenticate
            let mut response = Response::with((status::Unauthorized, "Unauthorized"));
            response.headers.set_raw("WWW-Authenticate", vec![b"Basic".to_vec()]);
            return Ok(response);
        }
        
        next.process(req)
    }
}

// Note: These handlers could be (for example) iron-router instances

/// Iron Handler representing the V1 api for this application
struct ApiV1Handler;

impl Handler for ApiV1Handler {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "Handled by the V1 API")))
    }
}

/// Iron Handler representing the V2 api for this application
struct ApiV2Handler;

impl Handler for ApiV2Handler {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "Handled by the V2 API")))
    }
}

