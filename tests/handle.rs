extern crate iron;
extern crate iron_pipeline;
extern crate iron_test;

use iron::prelude::*;
use iron::{ Headers, status };

use iron_pipeline::prelude::*;

#[test]
fn test_handle() {
    
    // build a simple pipeline 
    let mut pipeline = Pipeline::new();
    
    pipeline.add(Handle(|_| {
        Ok(Response::with((status::Ok, "Hello, world")))
    }));
    
    let response = iron_test::request::head("http://localhost/", Headers::new(), &pipeline).unwrap();
    assert_eq!(response.status, Some(status::Ok));
    
    let response_body = iron_test::response::extract_body_to_bytes(response);
    assert_eq!(response_body, b"Hello, world");
}

#[test]
fn test_handle_next() {
    
    // build a simple pipeline, with a middleware which modifies the response status code
    let mut pipeline = Pipeline::new();
    
    pipeline.add(HandleNext(|req, next| {
        let mut response = next.process(req).unwrap();
        response.status = Some(status::InternalServerError); // Overwrite the status
        Ok(response)
    }));
    
    pipeline.add(Handle(|_| {
        Ok(Response::with(status::Ok))
    }));
    
    let response = iron_test::request::head("http://localhost/", Headers::new(), &pipeline).unwrap();
    assert_eq!(response.status, Some(status::InternalServerError));
}