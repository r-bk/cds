use crate::{arrayvec::ArrayVec, len::LengthType, mem::SpareMemoryPolicy};
use core::cmp::{Eq, PartialEq};

impl<T, L, U, SM, const C: usize, const N: usize> PartialEq<&'_ [U; N]> for ArrayVec<T, C, L, SM>
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

impl<T, L, U, SM, const C: usize, const N: usize> PartialEq<[U; N]> for ArrayVec<T, C, L, SM>
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

impl<T, L, U, SM, const C: usize> PartialEq<&'_ [U]> for ArrayVec<T, C, L, SM>
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

impl<T, L, U, SM, const C: usize> PartialEq<[U]> for ArrayVec<T, C, L, SM>
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

impl<T, U, LT, LU, SMT, SMU, const C: usize, const N: usize> PartialEq<ArrayVec<U, N, LU, SMU>>
    for ArrayVec<T, C, LT, SMT>
where
    T: PartialEq<U>,
    LT: LengthType,
    LU: LengthType,
    SMT: SpareMemoryPolicy<T>,
    SMU: SpareMemoryPolicy<U>,
{
    #[inline]
    fn eq(&self, other: &ArrayVec<U, N, LU, SMU>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, LT, LU, SMT, SMU, const C: usize, const N: usize> PartialEq<&'_ ArrayVec<U, N, LU, SMU>>
    for ArrayVec<T, C, LT, SMT>
where
    T: PartialEq<U>,
    LT: LengthType,
    LU: LengthType,
    SMT: SpareMemoryPolicy<T>,
    SMU: SpareMemoryPolicy<U>,
{
    #[inline]
    fn eq(&self, other: &&'_ ArrayVec<U, N, LU, SMU>) -> bool {
        self[..] == other[..]
    }
}

impl<T: Eq, L: LengthType, SM: SpareMemoryPolicy<T>, const C: usize> Eq for ArrayVec<T, C, L, SM> {}

#[cfg(test)]
mod testing {
    use crate as cds;
    use cds::{array_vec, arrayvec::ArrayVec};

    type A = ArrayVec<u64, 7>;

    #[test]
    fn test_eq_arr_ref() {
        let a = A::from_iter(0..3);
        assert!(a == &[0, 1, 2]);
        assert!(a != &[0, 1, 2, 3]);
    }

    #[test]
    fn test_eq_arr() {
        let a = A::from_iter(1..3);
        assert!(a == [1, 2]);
        assert!(a != [1]);
    }

    #[test]
    fn test_eq_slice_ref() {
        let a = A::from_iter((100..102).rev());
        assert!(a == [101u64, 100].as_ref());
        assert!(a != [100].as_ref());
    }

    #[test]
    fn test_eq_slice() {
        let a = A::from_iter(3..5);
        let arr1: [u64; 2] = [3, 4];
        let arr2: [u64; 2] = [5, 6];
        assert!(a == arr1[..]);
        assert!(a != arr2[..]);
    }

    #[test]
    fn test_eq_av_ref() {
        let a = A::from_iter(0..2);
        let b = array_vec![2; u64; 0, 1];
        assert!(a == &b);
        assert!(a != &array_vec![7; u64]);
    }

    #[test]
    fn test_eq_av() {
        let a = A::from_iter(0..2);
        let b = array_vec![2; u64; 0, 1];
        assert!(a == b);
        assert!(a != array_vec![7; u64]);
    }
}
