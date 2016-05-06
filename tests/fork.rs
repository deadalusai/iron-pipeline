extern crate iron;
extern crate iron_pipeline;
extern crate iron_test;

use iron::prelude::*;
use iron::{ Headers, status };
use iron::method::{ Method };

use iron_pipeline::prelude::*;

#[test]
fn test_fork_when() {
    
    // build a pipeline which forks on HEAD requests
    let mut pipeline = Pipeline::new();
    pipeline.add(fork_when(|req| req.method == Method::Head, |posts| {
        posts.add(Handle(|_| {
            Ok(Response::with(status::Ok))
        }))
    }));
    pipeline.add(Handle(|_| {
        Ok(Response::with(status::InternalServerError))
    }));
    
    // test HEAD
    let response = iron_test::request::head("http://localhost/", Headers::new(), &pipeline).unwrap();
    assert_eq!(response.status, Some(status::Ok));
    
    // test all other methods
    let response = iron_test::request::get("http://localhost/", Headers::new(), &pipeline).unwrap();
    assert_eq!(response.status, Some(status::InternalServerError));
}

#[test]
fn test_fork_when_path() {
    
    // build a pipeline which forks when the path starts with `/api/v2`
    let mut pipeline = Pipeline::new();
    pipeline.add(fork_when_path("/api/v2", |v2| {
        v2.add(Handle(|_| {
            Ok(Response::with(status::Ok))
        }))
    }));
    pipeline.add(Handle(|_| {
        Ok(Response::with(status::InternalServerError))
    }));
    
    // test /api/v2
    let response = iron_test::request::get("http://localhost/api/v2/example", Headers::new(), &pipeline).unwrap();
    assert_eq!(response.status, Some(status::Ok));
    
    // test all other paths
    let response = iron_test::request::get("http://localhost/api/v1/example", Headers::new(), &pipeline).unwrap();
    assert_eq!(response.status, Some(status::InternalServerError));
}