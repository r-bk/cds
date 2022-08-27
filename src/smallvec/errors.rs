//! `SmallVec` error types.

use crate::mem::errors::ReservationError;

use core::fmt::{Debug, Display, Formatter};

// ----------------------------------------------------------------------------

/// An error returned from [`try_insert`] method.
///
/// [`try_insert`]: super::SmallVec::try_insert
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InsertError {
    /// Requested index is out of bounds.
    InvalidIndex,

    /// Capacity reservation error occurred.
    ReservationError(ReservationError),
}

impl Display for InsertError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            InsertError::InvalidIndex => {
                write!(f, "smallvec insert error: invalid index")
            }
            InsertError::ReservationError(ref re) => match re {
                ReservationError::CapacityOverflow => {
                    write!(f, "smallvec insert error: capacity overflow")
                }
                ReservationError::AllocError { ref layout } => {
                    write!(
                        f,
                        "smallvec insert error: alloc error. layout {{ size: {}, align: {} }}",
                        layout.size(),
                        layout.align()
                    )
                }
            },
        }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for InsertError {}

// ----------------------------------------------------------------------------

#[cfg(all(test, feature = "std"))]
mod testing {
    use super::*;
    use alloc::alloc::Layout;

    #[test]
    fn test_insert_error_display() {
        let e = InsertError::InvalidIndex;
        assert_eq!(format!("{}", e), "smallvec insert error: invalid index");

        let e = InsertError::ReservationError(ReservationError::CapacityOverflow);
        assert_eq!(format!("{}", e), "smallvec insert error: capacity overflow");

        let e = InsertError::ReservationError(ReservationError::AllocError {
            layout: Layout::from_size_align(2, 4).unwrap(),
        });
        assert_eq!(
            format!("{}", e),
            "smallvec insert error: alloc error. layout { size: 2, align: 4 }"
        );
    }
}
