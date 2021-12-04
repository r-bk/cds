use crate::{arrayvec::ArrayVec, mem::SpareMemoryPolicy};
use core::{iter::IntoIterator, slice};

impl<'a, T, SM, const C: usize> IntoIterator for &'a ArrayVec<T, SM, C>
where
    SM: SpareMemoryPolicy<T>,
{
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, SM, const C: usize> IntoIterator for &'a mut ArrayVec<T, SM, C>
where
    SM: SpareMemoryPolicy<T>,
{
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
