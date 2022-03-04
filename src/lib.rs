//! *cds* is a Collection of Data Structures.
//!
//! *cds* implements handy data structures and their associated algorithms.
//!
//!
//! # Design and Implementation Principles
//!
//! 1. **Tested** - for both correctness and performance
//! 2. **Fast** - even if it requires `unsafe` Rust
//! 3. **Secure** - do not unnecessarily hold a copy of (possibly sensitive) user data;
//!    wipe unused memory
//! 4. **No malloc** - avoid dynamic memory allocation where possible
//! 5. **Compact** - allow small memory footprint
//!
//!
//! # Fixed-Capacity Data Structures
//!
//! Fixed-capacity data structures can be allocated on stack.
//! They do not allocate memory on the heap, and their capacity cannot be dynamically changed.
//!
//! * [`ArrayVec`] - a vector-like array
//!
//!
//! # Crate Features
//!
//! * `arrayvec` - enables [`ArrayVec`]
//! * `alloc` - enables usage of the standard [`alloc`] crate
//! * `std` - implies `alloc` and enables implementation of [`std`] traits which are not available
//!   in [`core`]. Without this feature the library is [`no_std`].
//!
//! [`ArrayVec`]: crate::arrayvec::ArrayVec
//! [`no_std`]: https://docs.rust-embedded.org/book/intro/no-std.html
//!

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "arrayvec")]
#[cfg_attr(docsrs, doc(cfg(feature = "arrayvec")))]
pub mod arrayvec;

pub mod errors;

pub mod len;
pub mod mem;

pub(crate) mod sealed;

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub(crate) mod testing;
