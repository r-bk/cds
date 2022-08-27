use crate::{
    len::LengthType,
    mem::{errors::ReservationError, SpareMemoryPolicy},
    smallvec::{clone_from_slice_unchecked, SmallVec, NOHAE},
};
use core::{convert::TryFrom, mem, ptr};

impl<T, L, SM, const C: usize> TryFrom<&[T]> for SmallVec<T, C, L, SM>
where
    T: Clone,
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    type Error = ReservationError;

    #[inline]
    fn try_from(s: &[T]) -> Result<Self, Self::Error> {
        let mut tmp = Self::new();
        let (len, p) = tmp.try_reserve_exact_impl::<NOHAE>(s.len())?;
        unsafe {
            clone_from_slice_unchecked(s, len, p);
        }
        Ok(tmp)
    }
}

impl<T, L, SM, const C: usize> TryFrom<&mut [T]> for SmallVec<T, C, L, SM>
where
    T: Clone,
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    type Error = ReservationError;

    #[inline]
    fn try_from(s: &mut [T]) -> Result<Self, Self::Error> {
        let mut tmp = Self::new();
        let (len, p) = tmp.try_reserve_exact_impl::<NOHAE>(s.len())?;
        unsafe {
            clone_from_slice_unchecked(s, len, p);
        }
        Ok(tmp)
    }
}

impl<T, L, SM, const C: usize, const N: usize> TryFrom<[T; N]> for SmallVec<T, C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    type Error = ReservationError;

    #[inline]
    fn try_from(a: [T; N]) -> Result<Self, Self::Error> {
        let mut tmp = Self::new();
        let (len, p) = tmp.try_reserve_exact_impl::<NOHAE>(N)?;
        unsafe {
            ptr::copy_nonoverlapping(a.as_ptr(), p, N);
            len.set(N);
        }
        mem::forget(a);
        Ok(tmp)
    }
}

#[cfg(all(test, feature = "std"))]
mod testing {
    use crate as cds;
    use cds::{
        smallvec::SmallVec,
        testing::dropped::{Dropped, Track},
    };

    #[test]
    fn test_try_from_slice() {
        const TS: usize = 16;
        type SV<'a> = SmallVec<Dropped<'a, TS>, 2>;
        let t = Track::new();
        let v = Vec::from_iter(t.take(5));
        t.dropped_range(0..0); // empty range

        let sv = SV::try_from(v.as_slice()).unwrap();
        t.dropped_range(0..0);

        drop(v);
        t.dropped_range(0..5);

        drop(sv);
        t.dropped_range(0..10);
    }

    #[test]
    fn test_try_from_mut_slice() {
        const TS: usize = 16;
        type SV<'a> = SmallVec<Dropped<'a, TS>, 2>;
        let t = Track::new();
        let mut v = Vec::from_iter(t.take(5));
        t.dropped_range(0..0); // empty range

        let sv = SV::try_from(v.as_mut_slice()).unwrap();
        t.dropped_range(0..0);

        drop(v);
        t.dropped_range(0..5);

        drop(sv);
        t.dropped_range(0..10);
    }

    #[test]
    fn test_try_from_arr() {
        let arr = [5, 6, 7];
        type SV = SmallVec<usize, 2>;
        let v = SV::try_from(arr).unwrap();
        assert_eq!(v, [5, 6, 7]);
    }
}
