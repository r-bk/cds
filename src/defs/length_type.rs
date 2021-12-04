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

        /// Creates a new length-type value from `usize`.
        fn new(value: usize) -> Self;

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
/// *cds* fixed capacity collections allow customization of the type used to track the collection
/// length. This allows more compact representation of a collection type, especially
/// when low capacity is required.
///
/// Every length-type having `N` bits in the underlying type supports collections with capacity of
/// up to `2 ^ N - 1` elements.
pub trait LengthType: LengthTypeBase {}

// ------------------------------------------------------------------------------------------------

macro_rules! length_type {
    ($(#[$outer:meta])* $N:ident, $U:ty) => {
        $(#[$outer])*
        #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
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

            #[inline]
            fn new(val: usize) -> $N {
                debug_assert!(val <= Self::MAX);
                $N(val as $U)
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
                debug_assert!(self.0 as usize + rhs <= <Self as LengthTypeBase>::MAX);
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
