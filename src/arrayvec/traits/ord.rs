use crate::{arrayvec::ArrayVec, mem::SpareMemoryPolicy};
use core::cmp::{Ord, Ordering, PartialOrd};

impl<T, SM, const C: usize> PartialOrd for ArrayVec<T, SM, C>
where
    T: PartialOrd,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

impl<T, SM, const C: usize> Ord for ArrayVec<T, SM, C>
where
    T: Ord,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}
