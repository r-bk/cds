//! `SmallVec` error types.

use core::{
    alloc::Layout,
    fmt::{Debug, Display, Formatter},
};

// ----------------------------------------------------------------------------

/// An error returned when capacity reservation fails.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ReservationError {
    /// `SmallVec` capacity overflow.
    ///
    /// Is returned when the `SmallVec` generic parameter `L`, which is the
    /// small-vector's length type, cannot represent the resulting overall capacity.
    CapacityOverflow,

    /// Allocation error.
    ///
    /// Is returned when the underlying memory allocator fails to fulfill
    /// an allocation request.
    ///
    /// See [`alloc`] for more information.
    ///
    /// [`alloc`]: alloc::alloc::GlobalAlloc::alloc
    AllocError {
        /// The layout passed to the underlying allocator.
        layout: Layout,
    },
}

impl Display for ReservationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            ReservationError::CapacityOverflow => {
                write!(f, "smallvec reservation error: capacity overflow")
            }
            ReservationError::AllocError { ref layout } => {
                write!(
                    f,
                    "smallvec reservation error: alloc error. layout {{ size: {}, align: {} }}",
                    layout.size(),
                    layout.align()
                )
            }
        }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for ReservationError {}

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

    #[test]
    fn test_reservation_error_display() {
        let e = ReservationError::CapacityOverflow;
        assert_eq!(
            e.to_string(),
            "smallvec reservation error: capacity overflow"
        );

        let layout = alloc::alloc::Layout::from_size_align(100, 8).unwrap();
        let e = ReservationError::AllocError { layout };
        assert_eq!(
            e.to_string(),
            "smallvec reservation error: alloc error. layout { size: 100, align: 8 }"
        )
    }

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
