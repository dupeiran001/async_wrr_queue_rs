#![doc = include_str!("../README.md")]

mod wrr_queue;

mod instance;

pub(crate) mod consts;

#[cfg(all(feature = "tokio", feature = "blocking"))]
compile_error!(
    "feature 'tokio' and 'blocking' cannot be enabled together, consider disable default features"
);

#[cfg(not(any(feature = "tokio", feature = "blocking")))]
compile_error!("feature 'tokio' or 'blocking' must be enabled");

pub use instance::Instance;
pub use wrr_queue::WrrQueue;
