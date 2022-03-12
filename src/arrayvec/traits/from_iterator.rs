use crate::{arrayvec::ArrayVec, len::LengthType, mem::SpareMemoryPolicy};
use core::iter::{FromIterator, IntoIterator};

impl<T, L, SM, const C: usize> FromIterator<T> for ArrayVec<T, C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    /// Creates an `ArrayVec` from an iterator.
    ///
    /// # Panics
    ///
    /// This method panics if the iterator yields more than [`CAPACITY`] elements.
    ///
    /// [`CAPACITY`]: ArrayVec::CAPACITY
    #[inline]
    fn from_iter<I>(i: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self::try_from_iter(i).expect("insufficient capacity")
    }
}

#[cfg(test)]
mod testing {
    use crate as cds;
    use cds::{
        arrayvec::ArrayVec,
        testing::dropped::{Dropped, Track},
    };

    #[test]
    fn test_from_iter() {
        type A<'a> = ArrayVec<Dropped<'a, 16>, 16>;
        let t = Track::<16>::new();
        let a = A::from_iter(t.take(5));
        assert_eq!(t.n_allocated(), 5);
        assert!(t.dropped_range(0..0)); // empty range

        drop(a);
        assert_eq!(t.n_allocated(), 0);
        assert!(t.dropped_range(0..=4));
    }

    #[test]
    fn test_from_iter_copy() {
        type A = ArrayVec<usize, 8>;
        let a = A::from_iter(0..5);
        assert_eq!(a, [0, 1, 2, 3, 4]);
    }

    #[test]
    #[should_panic]
    fn test_from_iter_panics_on_capacity_error() {
        type A = ArrayVec<usize, 8>;
        let _a = A::from_iter(0..9);
    }
}
