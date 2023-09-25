use crate::{len::LengthType, mem::SpareMemoryPolicy, smallvec::SmallVec};
use core::cmp::{Eq, PartialEq};

impl<T, L, U, SM, const C: usize, const N: usize> PartialEq<&'_ [U; N]> for SmallVec<T, C, L, SM>
where
    T: PartialEq<U>,
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn eq(&self, other: &&'_ [U; N]) -> bool {
        self[..] == other[..]
    }
}

impl<T, L, U, SM, const C: usize, const N: usize> PartialEq<[U; N]> for SmallVec<T, C, L, SM>
where
    T: PartialEq<U>,
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn eq(&self, other: &[U; N]) -> bool {
        self[..] == other[..]
    }
}

impl<T, L, U, SM, const C: usize, const N: usize> PartialEq<[U; N]> for &SmallVec<T, C, L, SM>
where
    T: PartialEq<U>,
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn eq(&self, other: &[U; N]) -> bool {
        self[..] == other[..]
    }
}

impl<T, L, U, SM, const C: usize, const N: usize> PartialEq<[U; N]> for &mut SmallVec<T, C, L, SM>
where
    T: PartialEq<U>,
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn eq(&self, other: &[U; N]) -> bool {
        self[..] == other[..]
    }
}

impl<T, L, U, SM, const C: usize> PartialEq<&'_ [U]> for SmallVec<T, C, L, SM>
where
    T: PartialEq<U>,
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn eq(&self, other: &&'_ [U]) -> bool {
        self[..] == other[..]
    }
}

impl<T, L, U, SM, const C: usize> PartialEq<[U]> for SmallVec<T, C, L, SM>
where
    T: PartialEq<U>,
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn eq(&self, other: &[U]) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, LT, LU, SMT, SMU, const C: usize, const N: usize> PartialEq<SmallVec<U, N, LU, SMU>>
    for SmallVec<T, C, LT, SMT>
where
    T: PartialEq<U>,
    LT: LengthType,
    LU: LengthType,
    SMT: SpareMemoryPolicy<T>,
    SMU: SpareMemoryPolicy<U>,
{
    #[inline]
    fn eq(&self, other: &SmallVec<U, N, LU, SMU>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, LT, LU, SMT, SMU, const C: usize, const N: usize> PartialEq<&'_ SmallVec<U, N, LU, SMU>>
    for SmallVec<T, C, LT, SMT>
where
    T: PartialEq<U>,
    LT: LengthType,
    LU: LengthType,
    SMT: SpareMemoryPolicy<T>,
    SMU: SpareMemoryPolicy<U>,
{
    #[inline]
    fn eq(&self, other: &&'_ SmallVec<U, N, LU, SMU>) -> bool {
        self[..] == other[..]
    }
}

impl<T: Eq, L: LengthType, SM: SpareMemoryPolicy<T>, const C: usize> Eq for SmallVec<T, C, L, SM> {}

#[cfg(test)]
mod testing {
    use crate as cds;
    use cds::{small_vec, smallvec::SmallVec};

    type SV = SmallVec<u64, 7>;

    #[test]
    #[allow(clippy::op_ref)]
    fn test_eq_arr_ref() {
        let v = SV::from_iter(0..3);
        assert!(v == &[0, 1, 2]);
        assert!(v != &[0, 1, 2, 3]);
    }

    #[test]
    fn test_eq_arr() {
        let v = SV::from_iter(1..3);
        assert!(v == [1, 2]);
        assert!(v != [1]);
    }

    #[test]
    fn test_eq_slice_ref() {
        let v = SV::from_iter((100..102).rev());
        assert!(v == [101u64, 100].as_ref());
        assert!(v != [100].as_ref());
    }

    #[test]
    fn test_eq_slice() {
        let v = SV::from_iter(3..5);
        let arr1: [u64; 2] = [3, 4];
        let arr2: [u64; 2] = [5, 6];
        assert!(v == arr1[..]);
        assert!(v != arr2[..]);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn test_ref_eq_arr() {
        let v = SV::from_iter(3..5);
        let arr1: [u64; 2] = [3, 4];
        let arr2: [u64; 2] = [5, 6];
        assert!(&v == arr1);
        assert!(&v != arr2);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn test_eq_av_ref() {
        let a = SV::from_iter(0..2);
        let b = small_vec![2; u64; 0, 1];
        assert!(a == &b);
        assert!(a != &small_vec![7; u64]);
    }

    #[test]
    fn test_eq_av() {
        let a = SV::from_iter(0..2);
        let b = small_vec![2; u64; 0, 1];
        assert!(a == b);
        assert!(a != small_vec![7; u64]);
    }
}
