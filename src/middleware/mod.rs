//! Middleware for use in a `Pipeline`.
//!
//! # A note on `iron::middleware::Handler`
//!
//! All types which implement `iron::middleware::Handler` can also be used as iron-pipeline middleware.
//! However because the `Handler` trait does not understand the concept of "next" middleware,
//! it is generally only useful to put such handlers at the _end_ of a pipeline.
//!
//! For example, using `iron-router`:
//!
//! ```rustc
//! // Ensure the request is coming over HTTPS
//! pipeline.add(ProcessNext(|req, next| {
//!     if req.url.scheme != "https" {
//!         let new_url = build_https_redirect_url(&req.url);
//!         return Ok(Response::with((status::PermanentRedirect, Location(new_url))));
//!     }
//!     next.process(req)
//! }));
//! 
//! pipeline.add(router!(
//!     get  '/'           => entity::handle_get_all,
//!     get  '/entity/:id' => entity::handle_get_by_id,
//!     post '/entity/:id' => entity::handle_update_by_id,
//!     post '/entity/'    => entity::handle_create
//! ));
//! ```

pub mod fork;
pub mod process;
