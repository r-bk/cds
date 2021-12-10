use crate::{
    arrayvec::ArrayVec,
    defs::{LengthType, SpareMemoryPolicy},
};
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
    use crate::defs::{Uninitialized, Usize};
    use crate::testing::dropped::{Dropped, Track};

    #[test]
    fn test_clone_from() {
        type A<'a> = ArrayVec<Dropped<'a, 64>, Usize, Uninitialized, 8>;
        let track = Track::<64>::new();

        let mut a = A::try_from_iter(track.take(5)).unwrap();
        let b = A::try_from_iter(track.take(6)).unwrap();
        assert_eq!(track.n_allocated(), 11);
        assert_eq!(track.dropped(), []);

        a.clone_from(&b);

        assert_eq!(track.n_allocated(), 12);
        assert_eq!(track.dropped(), [0, 1, 2, 3, 4]);

        drop(b);

        assert_eq!(track.n_allocated(), 6);
        assert_eq!(track.dropped(), [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        drop(a);
        assert_eq!(track.n_allocated(), 0);
        assert_eq!(
            track.dropped(),
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
        )
    }

    #[test]
    fn test_clone() {
        type A<'a> = ArrayVec<Dropped<'a, 64>, Usize, Uninitialized, 8>;
        let track = Track::<64>::new();

        let a = A::try_from_iter(track.take(3)).unwrap();
        assert_eq!(track.n_allocated(), 3);

        let b = a.clone();
        assert_eq!(track.n_allocated(), 6);
        assert_eq!(track.dropped(), []);

        drop(a);
        assert_eq!(track.n_allocated(), 3);
        assert_eq!(track.dropped(), [0, 1, 2]);

        drop(b);
        assert_eq!(track.n_allocated(), 0);
        assert_eq!(track.dropped(), [0, 1, 2, 3, 4, 5]);
    }
}
