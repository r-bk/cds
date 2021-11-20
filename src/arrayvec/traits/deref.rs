use crate::ArrayVec;
use core::ops::{Deref, DerefMut};

impl<T, const C: usize> Deref for ArrayVec<T, C> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const C: usize> DerefMut for ArrayVec<T, C> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}
