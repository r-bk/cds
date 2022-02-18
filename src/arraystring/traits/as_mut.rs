use crate::{arraystring::ArrayString, len::LengthType, mem::SpareMemoryPolicy};
use core::convert::AsMut;

impl<L, SM, const C: usize> AsMut<str> for ArrayString<L, SM, C>
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
    use cds::array_str;

    #[test]
    fn test_as_mut_str() {
        let mut s = array_str![8; "cds"];
        let sl: &mut str = s.as_mut();
        assert_eq!(sl.to_ascii_uppercase(), "CDS");
    }
}
