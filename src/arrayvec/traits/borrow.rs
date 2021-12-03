use crate::arrayvec::ArrayVec;
use core::borrow::{Borrow, BorrowMut};

impl<T, const C: usize> Borrow<[T]> for ArrayVec<T, C> {
    #[inline]
    fn borrow(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, const C: usize> BorrowMut<[T]> for ArrayVec<T, C> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}
