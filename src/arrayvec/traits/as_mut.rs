use crate::{arrayvec::ArrayVec, defs::LengthType, mem::SpareMemoryPolicy};
use core::convert::AsMut;

impl<T, L, SM, const C: usize> AsMut<[T]> for ArrayVec<T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, L, SM, const C: usize> AsMut<ArrayVec<T, L, SM, C>> for ArrayVec<T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut ArrayVec<T, L, SM, C> {
        self
    }
}

#[cfg(test)]
mod testing {
    use crate as cds;
    use crate::{array_vec, arrayvec::ArrayVec, defs::Usize, mem::Uninitialized};
    use core::convert::AsMut;

    #[test]
    fn test_as_mut_slice() {
        let mut a = array_vec![3; u64; 1, 2, 3];
        let s: &mut [u64] = &mut [1, 2, 3];
        let a_s: &mut [u64] = a.as_mut();
        assert_eq!(a_s, s);
    }

    #[test]
    fn test_as_mut_av() {
        type A = ArrayVec<u64, Usize, Uninitialized, 3>;
        let mut a = A::try_from([1, 2, 3]).unwrap();
        let a_ref: &mut A = a.as_mut();
        assert_eq!((a_ref as *const A), (&a as *const A));
    }
}
