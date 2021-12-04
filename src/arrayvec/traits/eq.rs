use crate::{arrayvec::ArrayVec, defs::SpareMemoryPolicy};
use core::cmp::{Eq, PartialEq};

impl<T, U, SM, const C: usize, const N: usize> PartialEq<&'_ [U; N]> for ArrayVec<T, SM, C>
where
    T: PartialEq<U>,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn eq(&self, other: &&'_ [U; N]) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, SM, const C: usize, const N: usize> PartialEq<[U; N]> for ArrayVec<T, SM, C>
where
    T: PartialEq<U>,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn eq(&self, other: &[U; N]) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, SM, const C: usize> PartialEq<&'_ [U]> for ArrayVec<T, SM, C>
where
    T: PartialEq<U>,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn eq(&self, other: &&'_ [U]) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, SM, const C: usize> PartialEq<[U]> for ArrayVec<T, SM, C>
where
    T: PartialEq<U>,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn eq(&self, other: &[U]) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, SMT, SMU, const C: usize, const N: usize> PartialEq<ArrayVec<U, SMU, N>>
    for ArrayVec<T, SMT, C>
where
    T: PartialEq<U>,
    SMT: SpareMemoryPolicy<T>,
    SMU: SpareMemoryPolicy<U>,
{
    #[inline]
    fn eq(&self, other: &ArrayVec<U, SMU, N>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, SMT, SMU, const C: usize, const N: usize> PartialEq<&'_ ArrayVec<U, SMU, N>>
    for ArrayVec<T, SMT, C>
where
    T: PartialEq<U>,
    SMT: SpareMemoryPolicy<T>,
    SMU: SpareMemoryPolicy<U>,
{
    #[inline]
    fn eq(&self, other: &&'_ ArrayVec<U, SMU, N>) -> bool {
        self[..] == other[..]
    }
}

impl<T: Eq, SM: SpareMemoryPolicy<T>, const C: usize> Eq for ArrayVec<T, SM, C> {}
