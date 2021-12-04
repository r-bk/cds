use crate::{arrayvec::ArrayVec, errors::CapacityError, mem::SpareMemoryPolicy};
use core::{convert::TryFrom, mem, ptr};

impl<T, SM, const C: usize> TryFrom<&[T]> for ArrayVec<T, SM, C>
where
    T: Clone,
    SM: SpareMemoryPolicy<T>,
{
    type Error = CapacityError;

    #[inline]
    fn try_from(s: &[T]) -> Result<Self, Self::Error> {
        if s.len() > Self::CAPACITY {
            return Err(CapacityError {});
        }
        let mut tmp = Self::new();
        unsafe {
            tmp._clone_from_unchecked(s);
        }
        Ok(tmp)
    }
}

impl<T, SM, const C: usize> TryFrom<&mut [T]> for ArrayVec<T, SM, C>
where
    T: Clone,
    SM: SpareMemoryPolicy<T>,
{
    type Error = CapacityError;

    #[inline]
    fn try_from(s: &mut [T]) -> Result<Self, Self::Error> {
        if s.len() > Self::CAPACITY {
            return Err(CapacityError {});
        }
        let mut tmp = Self::new();
        unsafe {
            tmp._clone_from_unchecked(s);
        }
        Ok(tmp)
    }
}

impl<T, SM, const C: usize, const N: usize> TryFrom<[T; N]> for ArrayVec<T, SM, C>
where
    SM: SpareMemoryPolicy<T>,
{
    type Error = CapacityError;

    #[inline]
    fn try_from(a: [T; N]) -> Result<Self, Self::Error> {
        if N > Self::CAPACITY {
            return Err(CapacityError {});
        }
        let mut tmp = Self::new();
        unsafe {
            ptr::copy_nonoverlapping(a.as_ptr(), tmp.as_mut_ptr(), N);
        }
        tmp.len = N;
        mem::forget(a);
        Ok(tmp)
    }
}
