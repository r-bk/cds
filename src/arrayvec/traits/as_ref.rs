use crate::{arrayvec::ArrayVec, mem::SpareMemoryPolicy};
use core::convert::AsRef;

impl<T, SM, const C: usize> AsRef<[T]> for ArrayVec<T, SM, C>
where
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, SM, const C: usize> AsRef<ArrayVec<T, SM, C>> for ArrayVec<T, SM, C>
where
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn as_ref(&self) -> &ArrayVec<T, SM, C> {
        self
    }
}
