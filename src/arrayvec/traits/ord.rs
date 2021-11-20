use crate::ArrayVec;
use core::cmp::{Ord, Ordering, PartialOrd};

impl<T, const C: usize> PartialOrd for ArrayVec<T, C>
where
    T: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

impl<T, const C: usize> Ord for ArrayVec<T, C>
where
    T: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}
