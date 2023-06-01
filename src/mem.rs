//! Types and functions for dealing with memory.

mod policy;
pub use policy::*;

#[cfg(feature = "smallvec")]
pub(crate) mod alloc;

pub mod errors;
