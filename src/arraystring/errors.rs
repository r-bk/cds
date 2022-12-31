//! `ArrayString` error types.

use core::{
    clone::Clone,
    cmp::{Eq, Ord, PartialEq, PartialOrd},
    fmt::{Debug, Display, Formatter},
    hash::Hash,
    marker::Copy,
};

// ---------------------------------------------------------------------------

/// An error returned when there is no enough spare capacity.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct InsufficientCapacityError;

impl Display for InsufficientCapacityError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "arraystring insufficient capacity")
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for InsufficientCapacityError {}

// ---------------------------------------------------------------------------

/// An error returned from [`try_insert`] and [`try_insert_str`] methods.
///
/// [`try_insert`]: super::ArrayString::try_insert
/// [`try_insert_str`]: super::ArrayString::try_insert_str
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum InsertError {
    /// Index is out of bounds, or doesn't lie on a character boundary.
    InvalidIndex,

    /// There is no spare capacity to accommodate a new element.
    InsufficientCapacity,
}

impl Display for InsertError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let s = match *self {
            InsertError::InvalidIndex => "invalid index",
            InsertError::InsufficientCapacity => "insufficient capacity",
        };
        write!(f, "arraystring insert error: {s}")
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for InsertError {}

// ---------------------------------------------------------------------------

/// Index is invalid.
///
/// This error is returned when an index is out of bounds, or doesn't lie on a character boundary.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct IndexError;

impl core::fmt::Display for IndexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "arraystring index error: index is out of bounds or doesn't lie on character boundary"
        )
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for IndexError {}

#[cfg(all(test, feature = "std"))]
mod testing {
    use super::*;

    #[test]
    fn insufficient_capacity_error_display() {
        let e = InsufficientCapacityError {};
        let s = format!("{}", e);
        assert_eq!(s, "arraystring insufficient capacity");
    }

    #[test]
    fn insert_error_display() {
        let e = InsertError::InvalidIndex;
        let s = format!("{}", e);
        assert_eq!(s, "arraystring insert error: invalid index");

        let e = InsertError::InsufficientCapacity;
        let s = format!("{}", e);
        assert_eq!(s, "arraystring insert error: insufficient capacity");
    }

    #[test]
    fn index_error() {
        let e = IndexError {};
        let s = format!("{}", e);
        assert_eq!(
            s,
            "arraystring index error: index is out of bounds or doesn't lie on character boundary"
        );
    }
}
