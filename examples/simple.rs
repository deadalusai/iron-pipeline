extern crate iron;
extern crate iron_pipeline;

use iron::prelude::*;
use iron::status;
use iron_pipeline::prelude::*;

fn main() {
    let mut pipeline = Pipeline::new();
    
    // Inline "middleware" example using `HandleNext`.
    pipeline.add(HandleNext(|req, next| {
        log_request(req);
        let res = next.process(req);
        log_response(&res);
        res
    }));
    
    // Inline "handler" example using `Handle` 
    pipeline.add(Handle(|req| {
        Ok(Response::with((
            status::Ok,
            format!("Hello from iron-pipeline: {}", req.url)
        )))
    }));
    
    let port = 1337;
    let _listener =
        Iron::new(pipeline)
            .http(("localhost", port))
            .unwrap();
            
    println!("Listening on port {}", port);
}

fn log_request(req: &Request) {
    println!("{} {}", req.method, req.url);
}

fn log_response(result: &IronResult<Response>) {
    fn log_status(res: &Response) {
        match res.status {
            Some(status) => println!("{}", status),
            None         => println!("No response set")
        }
    }
    match result {
        &Ok(ref res) => {
            log_status(res);
        },
        &Err(ref err) => {
            println!("Error {}", err.error);
            log_status(&err.response);
        }
    } 
}