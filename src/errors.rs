//! Errors of the crate.

use core::fmt::Formatter;

/// An error returned when there is no free capacity in a collection.
#[derive(Debug, Copy, Clone)]
pub struct CapacityError;

impl core::fmt::Display for CapacityError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "insufficient capacity")
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for CapacityError {}
