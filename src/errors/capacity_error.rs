use core::{
    any::type_name,
    clone::Clone,
    fmt::{Debug, Display, Formatter},
    marker::Copy,
};

// ----------------------------------------------------------------------------

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

// ----------------------------------------------------------------------------

/// An error returned with a value when there is no free capacity in a collection.
#[derive(Copy, Clone)]
pub struct CapacityErrorVal<T>(pub T);

impl<T> Display for CapacityErrorVal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "insufficient capacity")
    }
}

impl<T> Debug for CapacityErrorVal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "CapacityErrorVal<{}>", type_name::<T>())
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl<T> std::error::Error for CapacityErrorVal<T> {}

// ----------------------------------------------------------------------------

#[cfg(all(test, feature = "std"))]
mod testing {
    use super::*;

    #[test]
    fn test_capacity_error_display() {
        let e = CapacityError {};
        let s = format!("{}", e);
        assert_eq!(s, "insufficient capacity");
    }

    #[test]
    fn test_capacity_error_debug() {
        let e = CapacityError {};
        let s = format!("{:?}", e);
        assert_eq!(s, "CapacityError");
    }

    #[test]
    fn test_capacity_error_copy() {
        let e = CapacityError {};
        let e2 = e;
        format!("{} {}", e, e2);
    }

    #[test]
    fn test_capacity_error_clone() {
        let e = CapacityError {};
        let e2 = e.clone();
        format!("{} {}", e, e2);
    }

    #[test]
    fn test_capacity_error_val_display() {
        let e = CapacityErrorVal::<u64>(17);
        let s = format!("{}", e);
        assert_eq!(s, "insufficient capacity")
    }

    #[test]
    fn test_capacity_error_val_debug() {
        let e = CapacityErrorVal::<u64>(717);
        let s = format!("{:?}", e);
        assert_eq!(s, "CapacityErrorVal<u64>")
    }

    #[test]
    fn test_capacity_error_val_clone() {
        let e = CapacityErrorVal::<String>("-11".into());
        let c = e.clone();
        assert_eq!(e.0, c.0);
        assert_eq!(e.0, "-11");
    }
}
