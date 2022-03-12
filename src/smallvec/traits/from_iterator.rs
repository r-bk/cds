use crate::{
    len::LengthType,
    mem::SpareMemoryPolicy,
    smallvec::{SmallVec, DOHAE},
};
use core::iter::{FromIterator, IntoIterator};

impl<T, L, SM, const C: usize> FromIterator<T> for SmallVec<T, C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    /// Creates a `SmallVec` from an iterator.
    ///
    /// # Panics
    ///
    /// This method panics on reservation errors. See [`reserve`] for more information.
    ///
    /// [`reserve`]: SmallVec::reserve
    #[inline]
    fn from_iter<I>(i: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self::try_from_iter_impl::<I, DOHAE>(i).expect("smallvec from_iter failed")
    }
}

#[cfg(test)]
mod testing {
    use crate as cds;
    use cds::{
        smallvec::SmallVec,
        testing::dropped::{Dropped, Track},
    };

    #[test]
    fn test_from_iter() {
        type SV<'a> = SmallVec<Dropped<'a, 16>, 16>;
        let t = Track::<16>::new();
        let v = SV::from_iter(t.take(5));
        assert_eq!(t.n_allocated(), 5);
        assert!(t.dropped_range(0..0)); // empty range

        drop(v);
        assert_eq!(t.n_allocated(), 0);
        assert!(t.dropped_range(0..=4));
    }

    #[test]
    fn test_from_iter_copy() {
        type SV = SmallVec<usize, 8>;
        let v = SV::from_iter(0..5);
        assert_eq!(v, [0, 1, 2, 3, 4]);
    }
}
