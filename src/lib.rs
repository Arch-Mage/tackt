#![warn(
    missing_docs,
    missing_copy_implementations,
    missing_debug_implementations,
    unused_qualifications,
    clippy::future_not_send,
    clippy::todo
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

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
