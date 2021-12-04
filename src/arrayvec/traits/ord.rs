use crate::{
    arrayvec::ArrayVec,
    defs::{LengthType, SpareMemoryPolicy},
};
use core::cmp::{Ord, Ordering, PartialOrd};

impl<T, L, SM, const C: usize> PartialOrd for ArrayVec<T, L, SM, C>
where
    T: PartialOrd,
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

impl<T, L, SM, const C: usize> Ord for ArrayVec<T, L, SM, C>
where
    T: Ord,
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}
