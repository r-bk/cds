use core::{
    alloc::Layout,
    fmt::{Debug, Display, Formatter},
};

/// An error returned when capacity reservation fails.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ReservationError {
    /// Collection capacity overflow.
    ///
    /// Is returned when a collection's length-type `L`
    /// cannot represent the resulting overall capacity.
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
                write!(f, "memory reservation error: capacity overflow")
            }
            ReservationError::AllocError { ref layout } => {
                write!(
                    f,
                    "memory reservation error: alloc error. layout {{ size: {}, align: {} }}",
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

#[cfg(all(test, feature = "std"))]
mod testing {
    use super::*;

    #[test]
    fn test_reservation_error_display() {
        let e = ReservationError::CapacityOverflow;
        assert_eq!(e.to_string(), "memory reservation error: capacity overflow");

        let layout = alloc::alloc::Layout::from_size_align(100, 8).unwrap();
        let e = ReservationError::AllocError { layout };
        assert_eq!(
            e.to_string(),
            "memory reservation error: alloc error. layout { size: 100, align: 8 }"
        )
    }
}
