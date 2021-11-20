use crate::ArrayVec;
use core::{
    ops::{Index, IndexMut},
    slice::SliceIndex,
};

impl<T, I: SliceIndex<[T]>, const C: usize> Index<I> for ArrayVec<T, C> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl<T, I: SliceIndex<[T]>, const C: usize> IndexMut<I> for ArrayVec<T, C> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut **self, index)
    }
}
