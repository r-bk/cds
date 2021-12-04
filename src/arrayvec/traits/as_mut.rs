use crate::{
    arrayvec::ArrayVec,
    defs::{LengthType, SpareMemoryPolicy},
};
use core::convert::AsMut;

impl<T, L, SM, const C: usize> AsMut<[T]> for ArrayVec<T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, L, SM, const C: usize> AsMut<ArrayVec<T, L, SM, C>> for ArrayVec<T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut ArrayVec<T, L, SM, C> {
        self
    }
}
