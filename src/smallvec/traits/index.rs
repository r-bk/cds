use crate::{len::LengthType, mem::SpareMemoryPolicy, smallvec::SmallVec};
use core::{
    ops::{Index, IndexMut},
    slice::SliceIndex,
};

impl<T, L, SM, I: SliceIndex<[T]>, const C: usize> Index<I> for SmallVec<T, C, L, SM>
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

impl<T, L, SM, I: SliceIndex<[T]>, const C: usize> IndexMut<I> for SmallVec<T, C, L, SM>
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
        len::U8,
        small_vec,
        smallvec::SmallVec,
        testing::dropped::{Dropped, Track},
    };

    #[test]
    fn test_index() {
        let v = small_vec![5; u64; 1, 2, 3, 4, 5];
        assert_eq!(v[1], 2);
        assert_eq!(v[..2], [1, 2]);
    }

    #[test]
    #[should_panic]
    fn test_index_panics() {
        let v = small_vec![5; u64; 1, 2, 3, 4, 5];
        #[allow(clippy::unnecessary_operation)]
        v[5];
    }

    #[test]
    #[should_panic]
    fn test_range_index_panics() {
        let v = small_vec![5; u64; 1, 2, 3, 4, 5];
        let _ = &v[3..7];
    }

    #[test]
    fn test_index_mut() {
        let mut v = small_vec![5; u64; 1, 2, 3, 4, 5];
        v[1] = 7;
        assert_eq!(v, [1, 7, 3, 4, 5]);

        let s = &mut v[1..=3];
        assert_eq!(s, [7, 3, 4]);
    }

    #[test]
    #[should_panic]
    fn test_index_mut_panics() {
        let mut v = small_vec![5; u64; 1, 2, 3, 4, 5];
        v[7] = 7;
    }

    #[test]
    #[should_panic]
    fn test_range_index_mut_panics() {
        let mut v = small_vec![5; u64; 1, 2, 3, 4, 5];
        let _ = &mut v[1..7];
    }

    #[test]
    fn test_index_mut_dropped() {
        type V<'a> = SmallVec<Dropped<'a, 5>, 5, U8>;
        let t = Track::new();
        let mut v = V::from_iter(t.take(3));
        assert!(t.dropped_indices(&[]));

        v[1] = t.alloc();
        assert!(t.dropped_indices(&[1]));

        v[1] = t.alloc();
        assert!(t.dropped_indices(&[1, 3]));

        drop(v);
        assert!(t.dropped_indices(&[0, 1, 2, 3, 4]));
    }
}
