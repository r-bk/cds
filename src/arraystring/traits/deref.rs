use crate::{arraystring::ArrayString, len::LengthType, mem::SpareMemoryPolicy};
use core::ops::{Deref, DerefMut};

impl<L, SM, const C: usize> Deref for ArrayString<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<L, SM, const C: usize> DerefMut for ArrayString<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_str()
    }
}

#[cfg(test)]
mod testing {
    use crate as cds;
    use cds::array_str;

    #[test]
    fn test_deref() {
        let s = array_str![8; "cds"];
        assert_eq!(&*s, "cds");
    }

    #[test]
    fn test_deref_mut() {
        let mut s = array_str![8; "cds"];
        (*s).make_ascii_uppercase();
        assert_eq!(&*s, "CDS");
    }
}
