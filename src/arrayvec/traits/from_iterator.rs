use crate::arrayvec::ArrayVec;
use core::iter::{FromIterator, IntoIterator};

impl<T, const C: usize> FromIterator<T> for ArrayVec<T, C> {
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
