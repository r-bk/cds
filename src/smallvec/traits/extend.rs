use crate::{
    len::LengthType,
    mem::SpareMemoryPolicy,
    smallvec::{SmallVec, DOHAE},
};
use core::iter::Extend;

impl<T, L, SM, const C: usize> Extend<T> for SmallVec<T, C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    /// Extend the small-vector with the contents of an iterator.
    ///
    /// # Panics
    ///
    /// This method panics on capacity reservation errors. See [`reserve`] for more information.
    ///
    /// [`reserve`]: SmallVec::reserve
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.try_extend_impl::<I, DOHAE>(iter)
            .expect("smallvec extend failed")
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
    fn test_extend() {
        let mut v = small_vec![6; usize; 9, 8, 7];
        assert_eq!(v, [9, 8, 7]);

        v.extend((4..7).rev());
        assert_eq!(v, [9, 8, 7, 6, 5, 4]);

        v.extend((1..4).rev());
        assert_eq!(v, [9, 8, 7, 6, 5, 4, 3, 2, 1]);
    }

    #[test]
    fn test_extend_dropped() {
        type ITEM<'a> = Dropped<'a, 512>;
        type SV<'a> = SmallVec<ITEM<'a>, 6, U8>;
        let t = Track::new();

        let mut v = SV::new();
        v.extend(t.take(5));
        assert_eq!(v.len(), 5);
        assert_eq!(v.is_local(), true);
        assert_eq!(v.capacity(), 6);
        assert!(t.dropped_range(0..0));

        v.extend(t.take(5));
        assert_eq!(v.len(), 10);
        assert_eq!(v.is_local(), false);
        assert_eq!(v.capacity(), 16);
        assert!(t.dropped_range(0..0));

        drop(v);
        assert!(t.dropped_range(0..10));
    }

    #[test]
    #[should_panic]
    fn test_extend_panics() {
        type ITEM<'a> = Dropped<'a, 512>;
        type SV<'a> = SmallVec<ITEM<'a>, 6, U8>;
        let t = Track::new();
        let mut v = SV::new();
        v.extend(t.take(300));
    }
}
