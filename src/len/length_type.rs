use core::{
    cmp::Ordering,
    fmt::{self, Display},
    ops::{AddAssign, SubAssign},
};

mod lt_private {
    use core::{
        fmt::Display,
        ops::{AddAssign, SubAssign},
    };

    pub trait LengthTypeBase:
        Copy
        + Send
        + Sync
        + Clone
        + PartialEq<usize>
        + PartialOrd<usize>
        + Eq
        + Ord
        + AddAssign<usize>
        + SubAssign<usize>
        + Display
    {
        /// The maximal length allowed by the underlying type.
        const MAX: usize;

        /// The underlying type.
        type U;

        /// Creates a new length-type value from `usize`.
        fn new(value: usize) -> Self;

        /// Returns `None` if calculation of `self + rhs` overflows, or the result otherwise
        fn checked_add_usize(&self, rhs: usize) -> Option<Self>;

        /// Returns the next power of two or Self::MAX if that overflows.
        fn next_power_of_two_or_max(&self) -> Self;

        /// Converts the underlying type to `usize`.
        fn as_usize(&self) -> usize;

        /// Sets the length value from `usize`.
        ///
        /// # Safety
        ///
        /// Implementations use debug-assertions to check that the new value is in bounds.
        fn set(&mut self, val: usize);
    }
}

use lt_private::LengthTypeBase;

/// A trait of custom length types.
///
/// *cds* collections allow customization of the type used to track the collection
/// length. This allows more compact representation of a collection type, especially
/// when low capacity is required.
///
/// Every length-type having `N` bits in the underlying type supports collections with capacity of
/// up to `2 ^ N - 1` elements.
///
/// Currently supported length types are: [`U8`], [`U16`], [`U32`], [`U64`] and [`Usize`].
///
/// # Examples
///
/// `LengthType` allows creation of very compact collections:
///
/// ```rust
/// # #[cfg(feature = "arrayvec")] {
/// use cds::{
///     arrayvec::ArrayVec,
///     len::U8
/// };
/// type A = ArrayVec<u8, 7, U8>;
/// assert_eq!(core::mem::size_of::<A>(), 8); // 7 element bytes + 1 length byte
/// # }
/// ```
///
/// In fixed-capacity collections, the requested capacity may not exceed the length-type's maximal
/// value:
///
/// ```should_panic
/// # #[cfg(feature = "arrayvec")] {
/// use cds::{
///     arrayvec::ArrayVec,
///     len::U8,
/// };
/// type A = ArrayVec<u8, 256, U8>; // U8::MAX(255) < CAPACITY(256)
/// let a = A::new();   // <-- this panics
/// # }
/// # #[cfg(not(feature = "arrayvec"))]
/// # panic!("just to fulfill the doc attribute")
/// ```
pub trait LengthType: LengthTypeBase {}

// ------------------------------------------------------------------------------------------------

macro_rules! length_type {
    ($(#[$outer:meta])* $N:ident, $U:ty) => {
        $(#[$outer])*
        #[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
        #[repr(transparent)]
        pub struct $N($U);

        impl PartialEq<usize> for $N {
            #[inline]
            fn eq(&self, other: &usize) -> bool {
                return self.0 as usize == *other
            }
        }

        impl PartialOrd<usize> for $N {
            #[inline]
            fn partial_cmp(&self, other: &usize) -> Option<Ordering> {
                (self.0 as usize).partial_cmp(other)
            }
        }

        impl LengthTypeBase for $N {
            const MAX: usize = <$U>::MAX as usize;
            type U = $U;

            #[inline]
            fn new(val: usize) -> $N {
                debug_assert!(val <= Self::MAX);
                $N(val as $U)
            }

            #[inline]
            fn checked_add_usize(&self, rhs: usize) -> Option<Self> {
                <$U>::try_from(rhs)
                    .ok()
                    .and_then(|v| self.0.checked_add(v))
                    .map(|u| $N(u))
            }

            #[inline]
            fn next_power_of_two_or_max(&self) -> Self {
                $N(self.0.checked_next_power_of_two().unwrap_or(<$U>::MAX))
            }

            #[inline]
            fn as_usize(&self) -> usize {
                self.0 as usize
            }

            #[inline]
            fn set(&mut self, val: usize) {
                debug_assert!(val <= Self::MAX);
                self.0 = val as $U;
            }
        }

        impl LengthType for $N {}

        impl AddAssign<usize> for $N {
            #[inline]
            fn add_assign(&mut self, rhs: usize) {
                debug_assert!((rhs <= <Self as LengthTypeBase>::MAX) &&
                                self.0.checked_add(rhs as $U).is_some());
                self.0 += rhs as $U
            }
        }

        impl SubAssign<usize> for $N {
            #[inline]
            fn sub_assign(&mut self, rhs: usize) {
                debug_assert!(self.0 as usize >= rhs);
                self.0 -= rhs as $U
            }
        }

        impl Display for $N {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
                Display::fmt(&(self.0 as $U), f)
            }
        }
    };
}

// ------------------------------------------------------------------------------------------------

length_type!(
    /// Length-type with underlying `u8`.
    U8,
    u8
);

length_type!(
    /// Length-type with underlying `u16`.
    U16,
    u16
);

length_type!(
    /// Length-type with underlying `u32`.
    U32,
    u32
);

length_type!(
    /// Length-type with underlying `u64`.
    U64,
    u64
);

length_type!(
    /// Length-type with underlying `usize`.
    Usize,
    usize
);

// ------------------------------------------------------------------------------------------------

#[cfg(all(test, feature = "std"))]
mod testing {
    use super::*;

    macro_rules! check_display {
        ($LT:expr, $C:literal) => {
            let lt = $LT($C);
            let s = format!("{}", lt);
            assert_eq!(s, $C.to_string());
        };
    }

    #[test]
    fn test_display() {
        check_display!(U8, 7);
        check_display!(U16, 707);
        check_display!(U32, 70707);
        check_display!(U64, 7070707);
        check_display!(Usize, 707070707);
    }
}
