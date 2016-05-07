//! Middleware for use in a `Pipeline`.
//!
//! # A note on `iron::middleware::Handler`
//!
//! All types which implement `iron::middleware::Handler` can also be used
//! as iron-pipeline middleware. However because the `Handler` trait does
//! not understand the concept of "next" middleware, it is generally only
//! useful to put such handlers at the _end_ of a pipeline.

pub mod fork;
pub mod handle;
