use crate::{arrayvec::ArrayVec, errors::CapacityError, len::LengthType, mem::SpareMemoryPolicy};
use core::{convert::TryFrom, mem, ptr};

impl<T, L, SM, const C: usize> TryFrom<&[T]> for ArrayVec<T, C, L, SM>
where
    T: Clone,
    L: LengthType,
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

impl<T, L, SM, const C: usize> TryFrom<&mut [T]> for ArrayVec<T, C, L, SM>
where
    T: Clone,
    L: LengthType,
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

impl<T, L, SM, const C: usize, const N: usize> TryFrom<[T; N]> for ArrayVec<T, C, L, SM>
where
    L: LengthType,
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
            tmp.set_len(N);
        }
        mem::forget(a);
        Ok(tmp)
    }
}

#[cfg(test)]
mod testing {
    use crate as cds;
    use cds::{arrayvec::ArrayVec, errors::CapacityError};
    type A = ArrayVec<u64, 7>;

    #[test]
    fn test_try_from_slice() {
        let a = A::try_from([1, 2, 3].as_ref()).unwrap();
        assert_eq!(a, [1, 2, 3]);
    }

    #[test]
    fn test_try_from_slice_err() {
        assert!(matches!(
            A::try_from([1, 2, 3, 4, 5, 6, 7, 8].as_ref()),
            Err(CapacityError)
        ));
    }

    #[test]
    fn test_try_from_mut_slice() {
        let a = A::try_from([1, 2, 3].as_mut()).unwrap();
        assert_eq!(a, [1, 2, 3]);
    }

    #[test]
    fn test_try_from_mut_slice_err() {
        assert!(matches!(
            A::try_from([1, 2, 3, 4, 5, 6, 7, 8].as_mut()),
            Err(CapacityError)
        ));
    }

    #[test]
    fn test_try_from_array() {
        let a = A::try_from([1, 2, 3]).unwrap();
        assert_eq!(a, [1, 2, 3]);
    }

    #[test]
    fn test_try_from_array_err() {
        assert!(matches!(
            A::try_from([1, 2, 3, 4, 5, 6, 7, 8]),
            Err(CapacityError)
        ));
    }
}
