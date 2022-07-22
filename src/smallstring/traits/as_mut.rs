use crate::{len::LengthType, mem::SpareMemoryPolicy, smallstring::SmallString};
use core::convert::AsMut;

impl<L, SM, const C: usize> AsMut<str> for SmallString<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut str {
        self.as_mut_str()
    }
}

#[cfg(test)]
mod testing {
    use crate as cds;
    use cds::small_str;

    #[test]
    fn test_as_mut_str() {
        let mut s = small_str![8; "cds"];
        let sl: &mut str = s.as_mut();
        assert_eq!(sl.to_ascii_uppercase(), "CDS");
    }
}
