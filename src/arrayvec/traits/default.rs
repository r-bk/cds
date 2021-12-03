use crate::arrayvec::ArrayVec;
use core::default::Default;

impl<T, const C: usize> Default for ArrayVec<T, C> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
