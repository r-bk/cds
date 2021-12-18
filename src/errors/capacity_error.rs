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
}
