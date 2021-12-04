use crate::{arrayvec::ArrayVec, defs::SpareMemoryPolicy};
use core::borrow::{Borrow, BorrowMut};

impl<T, SM, const C: usize> Borrow<[T]> for ArrayVec<T, SM, C>
where
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn borrow(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, SM, const C: usize> BorrowMut<[T]> for ArrayVec<T, SM, C>
where
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn borrow_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}
