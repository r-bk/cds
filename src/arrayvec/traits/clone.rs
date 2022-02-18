use crate::{arrayvec::ArrayVec, len::LengthType, mem::SpareMemoryPolicy};
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

#[cfg(test)]
mod testing {
    use super::*;
    use crate::len::Usize;
    use crate::mem::Uninitialized;
    use crate::testing::dropped::{Dropped, Track};

    #[test]
    fn test_clone_from() {
        type A<'a> = ArrayVec<Dropped<'a, 64>, Usize, Uninitialized, 8>;
        let track = Track::<64>::new();

        let mut a = A::try_from_iter(track.take(5)).unwrap();
        let b = A::try_from_iter(track.take(6)).unwrap();
        assert!(track.dropped_range(0..0)); // empty range

        a.clone_from(&b);

        assert_eq!(track.n_allocated(), 12);
        assert!(track.dropped_range(0..=4));

        drop(b);

        assert_eq!(track.n_allocated(), 6);
        assert!(track.dropped_range(0..=10));

        drop(a);
        assert_eq!(track.n_allocated(), 0);
        assert!(track.dropped_range(0..=16))
    }

    #[test]
    fn test_clone() {
        type A<'a> = ArrayVec<Dropped<'a, 64>, Usize, Uninitialized, 8>;
        let track = Track::<64>::new();

        let a = A::try_from_iter(track.take(3)).unwrap();
        assert_eq!(track.n_allocated(), 3);

        let b = a.clone();
        assert_eq!(track.n_allocated(), 6);
        assert!(track.dropped_range(0..0)); // empty range

        drop(a);
        assert_eq!(track.n_allocated(), 3);
        assert!(track.dropped_range(0..=2));

        drop(b);
        assert_eq!(track.n_allocated(), 0);
        assert!(track.dropped_range(0..=5));
    }
}
