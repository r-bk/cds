use crate::{arrayvec::ArrayVec, defs::SpareMemoryPolicy};
use core::iter::{FromIterator, IntoIterator};

impl<T, SM, const C: usize> FromIterator<T> for ArrayVec<T, SM, C>
where
    SM: SpareMemoryPolicy<T>,
{
    /// Creates an `ArrayVec` from an iterator.
    ///
    /// # Panics
    ///
    /// This method panics if the iterator yields more than [`CAPACITY`] elements.
    ///
    /// [`CAPACITY`]: ArrayVec::CAPACITY
    #[inline]
    fn from_iter<I>(i: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self::try_from_iter(i).expect("insufficient capacity")
    }
}
