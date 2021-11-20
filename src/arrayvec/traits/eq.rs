use crate::ArrayVec;
use core::cmp::{Eq, PartialEq};

impl<T, U, const C: usize, const N: usize> PartialEq<&'_ [U; N]> for ArrayVec<T, C>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &&'_ [U; N]) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, const C: usize, const N: usize> PartialEq<[U; N]> for ArrayVec<T, C>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &[U; N]) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, const C: usize> PartialEq<&'_ [U]> for ArrayVec<T, C>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &&'_ [U]) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, const C: usize> PartialEq<[U]> for ArrayVec<T, C>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &[U]) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, const C: usize, const N: usize> PartialEq<ArrayVec<U, N>> for ArrayVec<T, C>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &ArrayVec<U, N>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, const C: usize, const N: usize> PartialEq<&'_ ArrayVec<U, N>> for ArrayVec<T, C>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &&'_ ArrayVec<U, N>) -> bool {
        self[..] == other[..]
    }
}

impl<T: Eq, const C: usize> Eq for ArrayVec<T, C> {}
