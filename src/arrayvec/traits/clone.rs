use crate::{
    arrayvec::ArrayVec,
    defs::{LengthType, SpareMemoryPolicy},
};
use core::clone::Clone;

impl<T, L, SM, const C: usize> Clone for ArrayVec<T, L, SM, C>
where
    T: Clone,
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
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
