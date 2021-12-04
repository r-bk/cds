use crate::{arrayvec::ArrayVec, defs::SpareMemoryPolicy};
use core::convert::AsMut;

impl<T, SM, const C: usize> AsMut<[T]> for ArrayVec<T, SM, C>
where
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, SM, const C: usize> AsMut<ArrayVec<T, SM, C>> for ArrayVec<T, SM, C>
where
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut ArrayVec<T, SM, C> {
        self
    }
}
