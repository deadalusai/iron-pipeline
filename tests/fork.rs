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
    pipeline.add(Fork::when(|req| req.method == Method::Head, |posts| {
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
    pipeline.add(Fork::when_path("/api/v2", |v2| {
        v2.add(Handle(|req| {
            let body = req.url.path().join(":");
            Ok(Response::with((status::Ok, body)))
        }))
    }));
    pipeline.add(Handle(|_| {
        Ok(Response::with(status::InternalServerError))
    }));

    // test /api/v2
    let response = iron_test::request::get("http://localhost/api/v2/example/path", Headers::new(), &pipeline).unwrap();
    assert_eq!(response.status, Some(status::Ok));

    // test all other paths
    let response = iron_test::request::get("http://localhost/api/v1/example", Headers::new(), &pipeline).unwrap();
    assert_eq!(response.status, Some(status::InternalServerError));
}

#[test]
fn test_fork_when_path_strips_prefix() {

    let mut pipeline = Pipeline::new();

    // On path 1, print the URL path back to the response
    pipeline.add(Fork::when_path("/path/1", |app| {
        app.add(Handle(|req| {
            let body = req.url.path().join(":");
            Ok(Response::with(body))
        }))
    }));

    // On path 2, print the original URL path back to the response
    pipeline.add(Fork::when_path("/path/2", |app| {
        app.add(Handle(|req| {
            use iron_pipeline::middleware::fork::OriginalUrl;
            let original_url = req.extensions.get::<OriginalUrl>().unwrap();
            let body = original_url.path().join(":");
            Ok(Response::with(body))
        }))
    }));

    // Verify the path was truncated
    let response = iron_test::request::get("http://localhost/path/1/example/path", Headers::new(), &pipeline).unwrap();
    let result_body = iron_test::response::extract_body_to_bytes(response);
    assert_eq!(&result_body[..], &b"example:path"[..]);

    // Verify the original path is unchanged
    let response = iron_test::request::get("http://localhost/path/2/example/path", Headers::new(), &pipeline).unwrap();
    let result_body = iron_test::response::extract_body_to_bytes(response);
    assert_eq!(&result_body[..], &b"path:2:example:path"[..]);
}
