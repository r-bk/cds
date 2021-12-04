use crate::{
    arrayvec::ArrayVec,
    defs::{LengthType, SpareMemoryPolicy},
};
use core::iter::{FromIterator, IntoIterator};

impl<T, L, SM, const C: usize> FromIterator<T> for ArrayVec<T, L, SM, C>
where
    L: LengthType,
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
