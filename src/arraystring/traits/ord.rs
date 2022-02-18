use crate::{arraystring::ArrayString, len::LengthType, mem::SpareMemoryPolicy};
use core::cmp::{Ord, PartialOrd};
use std::cmp::Ordering;

impl<L, UL, SM, USM, const C: usize, const UC: usize> PartialOrd<ArrayString<UL, USM, UC>>
    for ArrayString<L, SM, C>
where
    L: LengthType,
    UL: LengthType,
    SM: SpareMemoryPolicy<u8>,
    USM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn partial_cmp(&self, other: &ArrayString<UL, USM, UC>) -> Option<Ordering> {
        PartialOrd::partial_cmp(self.as_str(), other.as_str())
    }
}

impl<L, SM, const C: usize> Ord for ArrayString<L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(self.as_str(), other.as_str())
    }
}

#[cfg(test)]
mod testing {
    use crate as cds;
    use cds::array_str;
    use std::cmp::Ordering;

    #[test]
    fn test_partial_ord() {
        let s1 = array_str![8; "cds"];
        let s2 = array_str![16; "cds"];
        let s3 = array_str![6; "CDS"];
        let s4 = array_str![3; "def"];
        assert_eq!(s1.partial_cmp(&s2), Some(Ordering::Equal));
        assert_eq!(s1.partial_cmp(&s3), Some(Ordering::Greater));
        assert_eq!(s1.partial_cmp(&s4), Some(Ordering::Less));
    }

    #[test]
    fn test_ord() {
        let s1 = array_str![8; "cds"];
        let s2 = array_str![8; "cds"];
        let s3 = array_str![8; "CDS"];
        let s4 = array_str![8; "def"];
        assert_eq!(s1.cmp(&s2), Ordering::Equal);
        assert_eq!(s1.cmp(&s3), Ordering::Greater);
        assert_eq!(s1.cmp(&s4), Ordering::Less);
    }
}
