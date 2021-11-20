use crate::ArrayVec;
use core::clone::Clone;

impl<T, const C: usize> Clone for ArrayVec<T, C>
where
    T: Clone,
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
