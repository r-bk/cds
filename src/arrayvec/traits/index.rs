use crate::{arrayvec::ArrayVec, defs::SpareMemoryPolicy};
use core::{
    ops::{Index, IndexMut},
    slice::SliceIndex,
};

impl<T, SM, I: SliceIndex<[T]>, const C: usize> Index<I> for ArrayVec<T, SM, C>
where
    SM: SpareMemoryPolicy<T>,
{
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl<T, SM, I: SliceIndex<[T]>, const C: usize> IndexMut<I> for ArrayVec<T, SM, C>
where
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut **self, index)
    }
}
