use crate::{arrayvec::ArrayVec, len::LengthType, mem::SpareMemoryPolicy};
use core::iter::Extend;

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
