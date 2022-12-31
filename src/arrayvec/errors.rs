//! `ArrayVec` error types.

use core::{
    any::type_name,
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
        write!(f, "arrayvec insufficient capacity")
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for InsufficientCapacityError {}

// ---------------------------------------------------------------------------

/// An error returned with a value when there is no enough spare capacity.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct InsufficientCapacityErrorVal<T>(pub T);

impl<T> Display for InsufficientCapacityErrorVal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "arrayvec insufficient capacity")
    }
}

impl<T> Debug for InsufficientCapacityErrorVal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "arrayvec::InsufficientCapacityErrorVal<{}>",
            type_name::<T>()
        )
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl<T> std::error::Error for InsufficientCapacityErrorVal<T> {}

// ---------------------------------------------------------------------------

/// An error returned from [`try_insert`] method.
///
/// [`try_insert`]: super::ArrayVec::try_insert
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum InsertError {
    /// Requested index is out of bounds
    InvalidIndex,

    /// There is no spare capacity to accommodate a new element.
    InsufficientCapacity,
}

impl core::fmt::Display for InsertError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let e = match *self {
            InsertError::InvalidIndex => "index is out of bounds",
            InsertError::InsufficientCapacity => "insufficient capacity",
        };
        write!(f, "arrayvec insert error: {e}")
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for InsertError {}

// ----------------------------------------------------------------------------

/// An error returned from [`try_insert_val`] method.
///
/// [`try_insert_val`]: super::ArrayVec::try_insert_val
#[derive(Copy, Clone)]
pub enum InsertErrorVal<T> {
    /// Requested index is out of bounds
    InvalidIndex(T),

    /// There is no spare capacity to accommodate a new element.
    InsufficientCapacity(T),
}

impl<T> InsertErrorVal<T> {
    /// Returns a reference to the value conveyed by the error.
    #[inline]
    pub fn value(&self) -> &T {
        match self {
            Self::InvalidIndex(v) => v,
            Self::InsufficientCapacity(v) => v,
        }
    }

    /// Returns the value conveyed by the error.
    #[inline]
    pub fn into_value(self) -> T {
        match self {
            Self::InvalidIndex(v) => v,
            Self::InsufficientCapacity(v) => v,
        }
    }
}

impl<T> Display for InsertErrorVal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let e = match self {
            Self::InvalidIndex(_) => "index is out of bounds",
            Self::InsufficientCapacity(_) => "insufficient capacity",
        };
        write!(f, "arrayvec insert error: {e}")
    }
}

impl<T> Debug for InsertErrorVal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let name = type_name::<T>();
        match self {
            Self::InvalidIndex(_) => write!(f, "InsertErrorVal<{name}>::InvalidIndex"),
            Self::InsufficientCapacity(_) => {
                write!(f, "InsertErrorVal<{name}>::InsufficientCapacity")
            }
        }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl<T> std::error::Error for InsertErrorVal<T> {}

// ----------------------------------------------------------------------------

#[cfg(all(test, feature = "std"))]
mod testing {
    use super::*;

    #[test]
    fn test_capacity_error_display() {
        let e = InsufficientCapacityError {};
        let s = format!("{}", e);
        assert_eq!(s, "arrayvec insufficient capacity");
    }

    #[test]
    fn test_capacity_error_debug() {
        let e = InsufficientCapacityError {};
        let s = format!("{:?}", e);
        assert_eq!(s, "InsufficientCapacityError");
    }

    #[test]
    fn test_capacity_error_copy() {
        let e = InsufficientCapacityError {};
        let e2 = e;
        format!("{} {}", e, e2);
    }

    #[test]
    fn test_capacity_error_clone() {
        let e = InsufficientCapacityError {};
        let e2 = e.clone();
        format!("{} {}", e, e2);
    }

    #[test]
    fn test_capacity_error_val_display() {
        let e = InsufficientCapacityErrorVal::<u64>(17);
        let s = format!("{}", e);
        assert_eq!(s, "arrayvec insufficient capacity")
    }

    #[test]
    fn test_capacity_error_val_debug() {
        let e = InsufficientCapacityErrorVal::<u64>(717);
        let s = format!("{:?}", e);
        assert_eq!(s, "arrayvec::InsufficientCapacityErrorVal<u64>")
    }

    #[test]
    fn test_capacity_error_val_clone() {
        let e = InsufficientCapacityErrorVal::<String>("-11".into());
        let c = e.clone();
        assert_eq!(e.0, c.0);
        assert_eq!(e.0, "-11");
    }

    #[test]
    fn test_insert_error_display() {
        let e = InsertError::InsufficientCapacity;
        let s = format!("{}", e);
        assert_eq!(s, "arrayvec insert error: insufficient capacity");

        let e = InsertError::InvalidIndex;
        let s = format!("{}", e);
        assert_eq!(s, "arrayvec insert error: index is out of bounds");
    }

    #[test]
    fn test_insert_error_debug() {
        let e = InsertError::InvalidIndex;
        let s = format!("{:?}", e);
        assert_eq!(s, "InvalidIndex");
    }

    #[test]
    fn test_insert_error_val_display() {
        let e = InsertErrorVal::<u64>::InvalidIndex(7);
        let s = format!("{}", e);
        assert_eq!(s, "arrayvec insert error: index is out of bounds");

        let e = InsertErrorVal::<u64>::InsufficientCapacity(17);
        let s = format!("{}", e);
        assert_eq!(s, "arrayvec insert error: insufficient capacity");
    }

    #[test]
    fn test_insert_error_val_debug() {
        let e = InsertErrorVal::<u64>::InvalidIndex(7);
        let s = format!("{:?}", e);
        assert_eq!(s, "InsertErrorVal<u64>::InvalidIndex");

        let e = InsertErrorVal::<u64>::InsufficientCapacity(17);
        let s = format!("{:?}", e);
        assert_eq!(s, "InsertErrorVal<u64>::InsufficientCapacity");
    }

    #[test]
    fn test_insert_error_val_copy() {
        let e = InsertErrorVal::<u64>::InvalidIndex(17);
        let e2 = e;
        assert_eq!(e.value(), e2.value());

        let e = InsertErrorVal::<u64>::InsufficientCapacity(717);
        let e2 = e;
        assert_eq!(e.value(), e2.value());
    }

    #[test]
    fn test_insert_error_val_clone() {
        let e = InsertErrorVal::<String>::InvalidIndex("17".into());
        let e2 = e.clone();
        assert_eq!(e.value(), e2.value());
        assert_eq!(e.value(), "17");

        let e = InsertErrorVal::<String>::InsufficientCapacity("717".into());
        let e2 = e.clone();
        assert_eq!(e.value(), e2.value());
        assert_eq!(e.value(), "717");
    }

    #[test]
    fn test_insert_error_val_into_value() {
        let e = InsertErrorVal::<String>::InvalidIndex("Hello, world!".into());
        let v = e.into_value();
        assert_eq!(v, "Hello, world!");

        let e = InsertErrorVal::<String>::InsufficientCapacity("Hello again!".into());
        let v = e.into_value();
        assert_eq!(v, "Hello again!");
    }
}
