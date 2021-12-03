use crate::arrayvec::ArrayVec;
use core::{iter::IntoIterator, slice};

impl<'a, T, const C: usize> IntoIterator for &'a ArrayVec<T, C> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, const C: usize> IntoIterator for &'a mut ArrayVec<T, C> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
