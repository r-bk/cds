use crate::{
    arrayvec::ArrayVec,
    defs::{LengthType, SpareMemoryPolicy},
};
use core::convert::AsRef;

impl<T, L, SM, const C: usize> AsRef<[T]> for ArrayVec<T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, L, SM, const C: usize> AsRef<ArrayVec<T, L, SM, C>> for ArrayVec<T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn as_ref(&self) -> &ArrayVec<T, L, SM, C> {
        self
    }
}
