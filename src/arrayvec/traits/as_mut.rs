use crate::arrayvec::ArrayVec;
use core::convert::AsMut;

impl<T, const C: usize> AsMut<[T]> for ArrayVec<T, C> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, const C: usize> AsMut<ArrayVec<T, C>> for ArrayVec<T, C> {
    #[inline]
    fn as_mut(&mut self) -> &mut ArrayVec<T, C> {
        self
    }
}
