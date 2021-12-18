use core::{
    any::type_name,
    clone::Clone,
    fmt::{Debug, Display, Formatter},
    marker::Copy,
};

// ----------------------------------------------------------------------------

/// An error returned from `try_insert` method.
#[derive(Debug, Copy, Clone)]
pub enum InsertError {
    /// Requested index is out of collection bounds.
    IndexOutOfBounds,

    /// There is no spare capacity to accommodate a new element.
    CapacityError,
}

impl core::fmt::Display for InsertError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match *self {
            InsertError::IndexOutOfBounds => write!(f, "index is out of bounds"),
            InsertError::CapacityError => write!(f, "insufficient capacity"),
        }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for InsertError {}

// ----------------------------------------------------------------------------

/// An error returned from `try_insert_val` method.
#[derive(Copy, Clone)]
pub enum InsertErrorVal<T> {
    /// Requested index is out of collection bounds.
    IndexOutOfBounds(T),

    /// There is no spare capacity to accommodate a new element.
    CapacityError(T),
}

impl<T> InsertErrorVal<T> {
    /// Returns a reference to the value conveyed by the error.
    #[inline]
    pub fn value(&self) -> &T {
        match self {
            Self::IndexOutOfBounds(v) => v,
            Self::CapacityError(v) => v,
        }
    }

    /// Returns the value conveyed by the error.
    #[inline]
    pub fn into_value(self) -> T {
        match self {
            Self::IndexOutOfBounds(v) => v,
            Self::CapacityError(v) => v,
        }
    }
}

impl<T> Display for InsertErrorVal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::IndexOutOfBounds(_) => write!(f, "index is out of bounds"),
            Self::CapacityError(_) => write!(f, "insufficient capacity"),
        }
    }
}

impl<T> Debug for InsertErrorVal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let name = type_name::<T>();
        match self {
            Self::IndexOutOfBounds(_) => write!(f, "InsertErrorVal<{}>::IndexOutOfBounds", name),
            Self::CapacityError(_) => write!(f, "InsertErrorVal<{}>::CapacityError", name),
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
    fn test_insert_error_display() {
        let e = InsertError::CapacityError;
        let s = format!("{}", e);
        assert_eq!(s, "insufficient capacity");

        let e = InsertError::IndexOutOfBounds;
        let s = format!("{}", e);
        assert_eq!(s, "index is out of bounds");
    }

    #[test]
    fn test_insert_error_debug() {
        let e = InsertError::IndexOutOfBounds;
        let s = format!("{:?}", e);
        assert_eq!(s, "IndexOutOfBounds");
    }

    #[test]
    fn test_insert_error_val_display() {
        let e = InsertErrorVal::<u64>::IndexOutOfBounds(7);
        let s = format!("{}", e);
        assert_eq!(s, "index is out of bounds");

        let e = InsertErrorVal::<u64>::CapacityError(17);
        let s = format!("{}", e);
        assert_eq!(s, "insufficient capacity");
    }

    #[test]
    fn test_insert_error_val_debug() {
        let e = InsertErrorVal::<u64>::IndexOutOfBounds(7);
        let s = format!("{:?}", e);
        assert_eq!(s, "InsertErrorVal<u64>::IndexOutOfBounds");

        let e = InsertErrorVal::<u64>::CapacityError(17);
        let s = format!("{:?}", e);
        assert_eq!(s, "InsertErrorVal<u64>::CapacityError");
    }

    #[test]
    fn test_insert_error_val_copy() {
        let e = InsertErrorVal::<u64>::IndexOutOfBounds(17);
        let e2 = e;
        assert_eq!(e.value(), e2.value());

        let e = InsertErrorVal::<u64>::CapacityError(717);
        let e2 = e;
        assert_eq!(e.value(), e2.value());
    }

    #[test]
    fn test_insert_error_val_clone() {
        let e = InsertErrorVal::<String>::IndexOutOfBounds("17".into());
        let e2 = e.clone();
        assert_eq!(e.value(), e2.value());
        assert_eq!(e.value(), "17");

        let e = InsertErrorVal::<String>::CapacityError("717".into());
        let e2 = e.clone();
        assert_eq!(e.value(), e2.value());
        assert_eq!(e.value(), "717");
    }

    #[test]
    fn test_insert_error_val_into_value() {
        let e = InsertErrorVal::<String>::IndexOutOfBounds("Hello, world!".into());
        let v = e.into_value();
        assert_eq!(v, "Hello, world!");

        let e = InsertErrorVal::<String>::CapacityError("Hello again!".into());
        let v = e.into_value();
        assert_eq!(v, "Hello again!");
    }
}
