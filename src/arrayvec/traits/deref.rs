use crate::{arrayvec::ArrayVec, defs::SpareMemoryPolicy};
use core::ops::{Deref, DerefMut};

impl<T, SM, const C: usize> Deref for ArrayVec<T, SM, C>
where
    SM: SpareMemoryPolicy<T>,
{
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, SM, const C: usize> DerefMut for ArrayVec<T, SM, C>
where
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}
