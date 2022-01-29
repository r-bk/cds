//! *cds* is a Collection of Data Structures.
//!
//! *cds* implements handy data structures and associated algorithms.
//!
//! It is driven by the following principles:
//!
//! 1. **Tested** - for both correctness and performance
//! 2. **Fast** - even if it requires to use unsafe Rust
//! 3. **Secure** - wipe unused memory to avoid unnecessarily holding
//!    a copy of possibly sensitive data
//! 4. Avoid dynamic memory allocation where possible
//!

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "arrayvec")]
#[cfg_attr(docsrs, doc(cfg(feature = "arrayvec")))]
pub mod arrayvec;

pub mod errors;

pub mod defs;

#[doc(inline)]
pub use defs::SpareMemoryPolicy;

#[doc(inline)]
pub use defs::LengthType;

pub(crate) mod sealed;

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub(crate) mod testing;
