use crate::{
    arrayvec::ArrayVec,
    defs::{LengthType, SpareMemoryPolicy},
};
use core::ops::Drop;

impl<T, L, SM, const C: usize> Drop for ArrayVec<T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn drop(&mut self) {
        self.truncate(0)
    }
}
