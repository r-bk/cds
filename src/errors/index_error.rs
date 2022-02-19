use core::{clone::Clone, fmt::Formatter, marker::Copy};

/// Index is invalid.
///
/// This error is returned when an index is out of bounds, or doesn't lie on a required boundary.
#[derive(Copy, Clone, Debug)]
pub struct IndexError;

impl core::fmt::Display for IndexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "index error")
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for IndexError {}
