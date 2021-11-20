use crate::ArrayVec;
use core::convert::AsRef;

impl<T, const C: usize> AsRef<[T]> for ArrayVec<T, C> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, const C: usize> AsRef<ArrayVec<T, C>> for ArrayVec<T, C> {
    #[inline]
    fn as_ref(&self) -> &ArrayVec<T, C> {
        self
    }
}
