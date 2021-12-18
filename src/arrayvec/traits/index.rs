use crate::{
    arrayvec::ArrayVec,
    defs::{LengthType, SpareMemoryPolicy},
};
use core::{
    ops::{Index, IndexMut},
    slice::SliceIndex,
};

impl<T, L, SM, I: SliceIndex<[T]>, const C: usize> Index<I> for ArrayVec<T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl<T, L, SM, I: SliceIndex<[T]>, const C: usize> IndexMut<I> for ArrayVec<T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut **self, index)
    }
}

#[cfg(test)]
mod testing {
    use crate as cds;
    use cds::{
        array_vec,
        arrayvec::ArrayVec,
        defs::{Uninitialized, U8},
        testing::dropped::{Dropped, Track},
    };

    #[test]
    fn test_index() {
        let a = array_vec![5; u64; 1, 2, 3, 4, 5];
        assert_eq!(a[1], 2);
        assert_eq!(a[..2], [1, 2]);
    }

    #[test]
    #[should_panic]
    fn test_index_panics() {
        let a = array_vec![5; u64; 1, 2, 3, 4, 5];
        a[5];
    }

    #[test]
    #[should_panic]
    fn test_range_index_panics() {
        let a = array_vec![5; u64; 1, 2, 3, 4, 5];
        let _ = &a[3..7];
    }

    #[test]
    fn test_index_mut() {
        let mut a = array_vec![5; u64; 1, 2, 3, 4, 5];
        a[1] = 7;
        assert_eq!(a, [1, 7, 3, 4, 5]);

        let s = &mut a[1..=3];
        assert_eq!(s, [7, 3, 4]);
    }

    #[test]
    #[should_panic]
    fn test_index_mut_panics() {
        let mut a = array_vec![5; u64; 1, 2, 3, 4, 5];
        a[7] = 7;
    }

    #[test]
    #[should_panic]
    fn test_range_index_mut_panics() {
        let mut a = array_vec![5; u64; 1, 2, 3, 4, 5];
        let _ = &mut a[1..7];
    }

    #[test]
    fn test_index_mut_dropped() {
        type A<'a> = ArrayVec<Dropped<'a, 5>, U8, Uninitialized, 5>;
        let t = Track::new();
        let mut a = A::from_iter(t.take(3));
        assert!(t.dropped_indices(&[]));

        a[1] = t.alloc();
        assert!(t.dropped_indices(&[1]));

        a[1] = t.alloc();
        assert!(t.dropped_indices(&[1, 3]));

        drop(a);
        assert!(t.dropped_indices(&[0, 1, 2, 3, 4]));
    }
}
