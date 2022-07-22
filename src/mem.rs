//! Types and functions for dealing with memory.

mod policy;
pub use policy::*;

#[cfg(any(feature = "smallvec", feature = "smallstring"))]
#[allow(dead_code)]
pub(crate) mod alloc;

pub mod errors;
