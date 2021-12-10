use crate::{
    arrayvec::ArrayVec,
    defs::{LengthType, SpareMemoryPolicy},
};
use core::convert::AsRef;

impl<T, L, SM, const C: usize> AsRef<[T]> for ArrayVec<T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, L, SM, const C: usize> AsRef<ArrayVec<T, L, SM, C>> for ArrayVec<T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn as_ref(&self) -> &ArrayVec<T, L, SM, C> {
        self
    }
}

#[cfg(test)]
mod testing {
    use crate as cds;
    use crate::{
        array_vec,
        arrayvec::ArrayVec,
        defs::{Uninitialized, Usize},
    };
    use core::convert::AsRef;

    #[test]
    fn test_as_ref_slice() {
        let a = array_vec![3; u64; 1, 2, 3];
        let s: &[u64] = &[1, 2, 3];
        let a_s: &[u64] = a.as_ref();
        assert_eq!(a_s, s);
    }

    #[test]
    fn test_as_ref_av() {
        type A = ArrayVec<u64, Usize, Uninitialized, 3>;
        let a = A::try_from([1, 2, 3]).unwrap();
        let a_ref: &A = a.as_ref();
        assert_eq!((a_ref as *const A), (&a as *const A));
    }
}
