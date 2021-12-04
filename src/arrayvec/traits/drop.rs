use crate::{arrayvec::ArrayVec, defs::SpareMemoryPolicy};
use core::ops::Drop;

impl<T, SM, const C: usize> Drop for ArrayVec<T, SM, C>
where
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn drop(&mut self) {
        self.truncate(0)
    }
}
