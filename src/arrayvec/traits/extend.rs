use crate::{
    arrayvec::ArrayVec,
    defs::{LengthType, SpareMemoryPolicy},
};
use core::iter::Extend;

struct Guard<'a, T, L, SM, const C: usize>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    av: &'a mut ArrayVec<T, L, SM, C>,
    len: usize,
}

impl<'a, T, L, SM, const C: usize> Drop for Guard<'a, T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn drop(&mut self) {
        unsafe { self.av.set_len(self.len) }
    }
}

impl<T, L, SM, const C: usize> Extend<T> for ArrayVec<T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    /// Extend the array-vector with the contents of an iterator.
    ///
    /// # Panics
    ///
    /// This method panics if extending the array-vector exceeds its capacity.
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for e in iter {
            self.push(e);
        }
    }
}
