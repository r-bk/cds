//! *cds* is a Collection of Optimized Data Structures.
//!
//! *cds* implements optimized data structures and their associated algorithms.
//!
//!
//! # Design and Implementation Principles
//!
//! 1. **Tested** - for both correctness and performance
//! 2. **Fast** - even if it requires `unsafe` Rust
//! 3. **Secure** - do not unnecessarily hold a copy of (possibly sensitive) user data;
//!    allow wiping of unused memory
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
//! * [`ArrayString`] - a string-like array
//!
//!
//! # Hybrid-Capacity Data Structures
//!
//! Hybrid-capacity data structures use both local and heap capacity.
//! They have some local capacity, which, if not exceeded, avoids heap allocation.
//! Once the amount of needed capacity exceeds the local capacity, a heap
//! allocation is made, existing data is copied to the heap, and the data structure continues its
//! operation from there.
//!
//! * [`SmallVec`] - a vector with “small size” optimization
//! * [`SmallString`] - a string with "small size" optimization
//!
//!
//! # Optional Features
//!
//! * `alloc` - enables usage of the standard [`alloc`] crate
//! * `std` - implies `alloc` and enables implementation of [`std`] traits which are not available
//!   in [`core`]. Without this feature the library is [`no_std`].
//! * `arrayvec` - enables [`ArrayVec`]
//! * `arraystring` - enables [`ArrayString`]
//! * `smallvec` - implies `alloc` and enables [`SmallVec`]
//! * `smallstring` - implies `alloc` and enables [`SmallString`]
//!
//! By default, all optional features are enabled. To build in `no_std` environment, or to avoid
//! compilation of unneeded functionality, disable default features and cherry pick the required
//! features explicitly.
//!
//! [`ArrayVec`]: crate::arrayvec::ArrayVec
//! [`ArrayString`]: crate::arraystring::ArrayString
//! [`SmallVec`]: crate::smallvec::SmallVec
//! [`SmallString`]: crate::smallstring::SmallString
//! [`no_std`]: https://docs.rust-embedded.org/book/intro/no-std.html
//!

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(any(feature = "alloc", feature = "test"))]
extern crate alloc;

#[cfg(feature = "arrayvec")]
#[cfg_attr(docsrs, doc(cfg(feature = "arrayvec")))]
pub mod arrayvec;

#[cfg(feature = "arraystring")]
#[cfg_attr(docsrs, doc(cfg(feature = "arraystring")))]
pub mod arraystring;

#[cfg(feature = "smallvec")]
#[cfg_attr(docsrs, doc(cfg(feature = "smallvec")))]
pub mod smallvec;

#[cfg(feature = "smallstring")]
#[cfg_attr(docsrs, doc(cfg(feature = "smallstring")))]
pub mod smallstring;

pub mod len;
pub mod mem;

pub(crate) mod sealed;

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub(crate) mod testing;
