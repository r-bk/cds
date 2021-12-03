//! *cds* is a Collection of Data Structures.
//!
//! *cds* is in development.
//! The version 0.0.1 is a crates.io placeholder.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "arrayvec")]
#[cfg_attr(docsrs, doc(cfg(feature = "arrayvec")))]
pub mod arrayvec;

#[cfg(feature = "arrayvec")]
#[doc(inline)]
pub use arrayvec::ArrayVec;

pub mod errors;

pub mod mem;

pub(crate) mod sealed;

#[cfg(test)]
pub(crate) mod testing;
