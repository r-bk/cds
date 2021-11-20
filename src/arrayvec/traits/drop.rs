use crate::ArrayVec;
use core::ops::Drop;

impl<T, const C: usize> Drop for ArrayVec<T, C> {
    #[inline]
    fn drop(&mut self) {
        self.truncate(0)
    }
}
