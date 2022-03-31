//! # tackt
//!
//! > HTTP router for [tower][tower] service.
//!
//! ## usage overview
//!
//! ```rust
//! use tackt::route;
//! use tackt::routes;
//!
//! #[route(GET, PUT: "entity" / id / "resource" / path*)]
//! async fn resource(
//!     req: http::Request<hyper::Body>,
//!     id: i32,
//!     path: String,
//! ) -> Result<http::Response<hyper::Body>, Box<dyn std::error::Error>> {
//!     let content = format!("resource: {id} {path}");
//!     let body = hyper::Body::from(content);
//!     let response = http::Response::new(body);
//!     Ok(response)
//! }
//!
//! let router = routes![resource];
//! // use the `router` in `hyper::service::make_service_fn`.
//! ```
//!
//! **NOTE**: `#[route]` attribute changes the function signature.
//!
//! ## route spec examples
//!
//! 1.  Empty
//!
//!     This spec will match exactly `"/"` on any methods.
//!
//!     ```rust,ignore
//!     #[route]
//!     ```
//!
//! 1.  Only methods
//!
//!     This spec will match exactly `"/"` only on `GET` or `PUT` request.
//!
//!     ```rust,ignore
//!     #[route(GET, PUT)]
//!     ```
//!
//! 1.  Only segments
//!
//!     This spec will match exactly `"/path/to/somewhere"` on any methods.
//!
//!     ```rust,ignore
//!     #[route("path" / "to" / "somewhere")]
//!     ```
//!
//! 1.  Methods and segments
//!
//!     This spec will match exactly `"/path/to/somewhere"` only on `GET` request.
//!
//!     ```rust,ignore
//!     #[route(GET: "path" / "to" / "somewhere")]
//!     ```
//!
//! ## route syntax:
//!
//! ```text
//! spec: methods ':' segments
//!     / methods
//!     / segments
//!     / empty
//!
//! methods: identifier [',' identifier]*
//!
//! segments: segment ['/' segment]* ['/' rest]
//!
//! segment: literal-str / identifier
//!
//! rest: identifier '*'
//!
//! empty:
//! ```
//!
//! [tower]: https://crates.io/crates/tower
#![warn(
    missing_docs,
    missing_copy_implementations,
    missing_debug_implementations,
    unused_qualifications
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod error;
mod func;
mod future;
mod macros;
mod mount;
mod or;
mod param;
mod request;
mod route;
mod router;
mod with;

#[cfg(feature = "hyper")]
#[cfg_attr(docsrs, doc(cfg(feature = "hyper")))]
mod make;

#[cfg(test)]
mod exec;

pub use error::Error;
pub use param::Param;
pub use route::Route;
pub use router::Router;

pub use request::MethodReq;
pub use request::PathReq;
pub use request::RemovePrefix;

pub use func::Func;
pub use mount::Mount;
pub use or::Or;

pub use http::Method;
pub use tower_service::Service;

#[cfg(feature = "macros")]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
pub use tackt_macros::route;
