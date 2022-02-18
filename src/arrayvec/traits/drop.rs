use crate::{arrayvec::ArrayVec, len::LengthType, mem::SpareMemoryPolicy};
use core::ops::Drop;

impl<T, L, SM, const C: usize> Drop for ArrayVec<T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn drop(&mut self) {
        self.truncate(0)
    }
}

#[cfg(test)]
mod testing {
    use crate as cds;
    use cds::{
        arrayvec::ArrayVec,
        len::Usize,
        mem::Uninitialized,
        testing::dropped::{Dropped, Track},
    };

    #[test]
    fn test_drop() {
        type A<'a> = ArrayVec<Dropped<'a, 16>, Usize, Uninitialized, 8>;
        let t = Track::<16>::new();
        let a = A::try_from_iter(t.take(5)).unwrap();
        assert_eq!(t.n_allocated(), 5);
        assert!(t.dropped_range(0..0)); // empty range

        drop(a);
        assert_eq!(t.n_allocated(), 0);
        assert!(t.dropped_range(0..=4));
    }

    #[test]
    fn test_drop_copy() {
        type A = ArrayVec<usize, Usize, Uninitialized, 7>;
        let a = A::try_from_iter((0..5).into_iter()).unwrap();
        drop(a);
    }
}
