use crate::{
    arrayvec::ArrayVec,
    defs::{LengthType, SpareMemoryPolicy},
};
use core::borrow::{Borrow, BorrowMut};

impl<T, L, SM, const C: usize> Borrow<[T]> for ArrayVec<T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn borrow(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, L, SM, const C: usize> BorrowMut<[T]> for ArrayVec<T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn borrow_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    use crate as cds;
    use crate::array_vec;

    #[test]
    fn test_borrow() {
        let a = array_vec![3; u64; 1, 2, 3];
        let a_s: &[u64] = a.borrow();
        assert_eq!(a_s, &[1, 2, 3]);
        assert_eq!(a_s.as_ptr(), a.as_ptr())
    }

    #[test]
    fn test_borrow_mut() {
        let mut a = array_vec![3; u64; 3, 2, 1];
        let a_s: &mut [u64] = a.borrow_mut();
        assert_eq!(a_s, &[3, 2, 1]);
        assert_eq!(a_s.as_mut_ptr(), a.as_mut_ptr());
    }
}
