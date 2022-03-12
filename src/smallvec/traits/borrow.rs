use crate::{len::LengthType, mem::SpareMemoryPolicy, smallvec::SmallVec};
use core::borrow::{Borrow, BorrowMut};

impl<T, L, SM, const C: usize> Borrow<[T]> for SmallVec<T, C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn borrow(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, L, SM, const C: usize> BorrowMut<[T]> for SmallVec<T, C, L, SM>
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
    use crate::small_vec;

    #[test]
    fn test_borrow() {
        let v = small_vec![3; u64; 1, 2, 3];
        let v_s: &[u64] = v.borrow();
        assert_eq!(v_s, &[1, 2, 3]);
        assert_eq!(v_s.as_ptr(), v.as_ptr())
    }

    #[test]
    fn test_borrow_mut() {
        let mut v = small_vec![3; u64; 3, 2, 1];
        let v_s: &mut [u64] = v.borrow_mut();
        assert_eq!(v_s, &[3, 2, 1]);
        assert_eq!(v_s.as_mut_ptr(), v.as_mut_ptr());
    }
}
