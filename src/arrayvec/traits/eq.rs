use crate::{
    arrayvec::ArrayVec,
    defs::{LengthType, SpareMemoryPolicy},
};
use core::cmp::{Eq, PartialEq};

impl<T, L, U, SM, const C: usize, const N: usize> PartialEq<&'_ [U; N]> for ArrayVec<T, L, SM, C>
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

impl<T, L, U, SM, const C: usize, const N: usize> PartialEq<[U; N]> for ArrayVec<T, L, SM, C>
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

impl<T, L, U, SM, const C: usize> PartialEq<&'_ [U]> for ArrayVec<T, L, SM, C>
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

impl<T, L, U, SM, const C: usize> PartialEq<[U]> for ArrayVec<T, L, SM, C>
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

impl<T, U, LT, LU, SMT, SMU, const C: usize, const N: usize> PartialEq<ArrayVec<U, LU, SMU, N>>
    for ArrayVec<T, LT, SMT, C>
where
    T: PartialEq<U>,
    LT: LengthType,
    LU: LengthType,
    SMT: SpareMemoryPolicy<T>,
    SMU: SpareMemoryPolicy<U>,
{
    #[inline]
    fn eq(&self, other: &ArrayVec<U, LU, SMU, N>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, LT, LU, SMT, SMU, const C: usize, const N: usize> PartialEq<&'_ ArrayVec<U, LU, SMU, N>>
    for ArrayVec<T, LT, SMT, C>
where
    T: PartialEq<U>,
    LT: LengthType,
    LU: LengthType,
    SMT: SpareMemoryPolicy<T>,
    SMU: SpareMemoryPolicy<U>,
{
    #[inline]
    fn eq(&self, other: &&'_ ArrayVec<U, LU, SMU, N>) -> bool {
        self[..] == other[..]
    }
}

impl<T: Eq, L: LengthType, SM: SpareMemoryPolicy<T>, const C: usize> Eq for ArrayVec<T, L, SM, C> {}
