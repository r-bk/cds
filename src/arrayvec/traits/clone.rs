use crate::{arrayvec::ArrayVec, mem::SpareMemoryPolicy};
use core::clone::Clone;

impl<T, SM, const C: usize> Clone for ArrayVec<T, SM, C>
where
    T: Clone,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn clone(&self) -> Self {
        let mut tmp = Self::new();
        tmp._clone_from(self);
        tmp
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.truncate(0);
        self._clone_from(source);
    }
}
