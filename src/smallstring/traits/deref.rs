use crate::{len::LengthType, mem::SpareMemoryPolicy, smallstring::SmallString};
use core::ops::{Deref, DerefMut};

impl<const C: usize, L, SM> Deref for SmallString<C, L, SM>
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

impl<const C: usize, L, SM> DerefMut for SmallString<C, L, SM>
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
    use cds::small_str;

    #[test]
    fn test_deref() {
        let s = small_str![8; "cds"];
        assert_eq!(&*s, "cds");
    }

    #[test]
    fn test_deref_mut() {
        let mut s = small_str![8; "cds"];
        (*s).make_ascii_uppercase();
        assert_eq!(&*s, "CDS");
    }
}
