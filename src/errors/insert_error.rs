use core::fmt::Formatter;

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
}
