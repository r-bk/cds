use crate::{arraystring::ArrayString, len::LengthType, mem::SpareMemoryPolicy};
use core::borrow::{Borrow, BorrowMut};

impl<L, SM, const C: usize> Borrow<str> for ArrayString<L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<L, SM, const C: usize> BorrowMut<str> for ArrayString<L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn borrow_mut(&mut self) -> &mut str {
        self.as_mut_str()
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    use crate as cds;
    use crate::array_str;

    #[test]
    fn test_borrow() {
        let s = array_str![3; "cds"];
        let s_b: &str = s.borrow();
        assert_eq!(s_b, "cds");
        assert_eq!(s_b.as_ptr(), s.as_ptr())
    }

    #[test]
    fn test_borrow_mut() {
        let mut s = array_str![5; "cds"];
        let s_b: &mut str = s.borrow_mut();
        assert_eq!(s_b, "cds");
        assert_eq!(s_b.as_mut_ptr(), s.as_mut_ptr());
    }
}
