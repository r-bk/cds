use crate::{arraystring::ArrayString, len::LengthType, mem::SpareMemoryPolicy};
use core::cmp::{Eq, PartialEq};

impl<'a, L, SM, const C: usize> PartialEq<&'a str> for ArrayString<L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        PartialEq::eq(self.as_str(), *other)
    }
}

impl<'a, L, SM, const C: usize> PartialEq<ArrayString<L, SM, C>> for &'a str
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn eq(&self, other: &ArrayString<L, SM, C>) -> bool {
        PartialEq::eq(*self, other.as_str())
    }
}

impl<L, SM, const C: usize> PartialEq<ArrayString<L, SM, C>> for str
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn eq(&self, other: &ArrayString<L, SM, C>) -> bool {
        PartialEq::eq(self, other.as_str())
    }
}

impl<L, SM, const C: usize> PartialEq<str> for ArrayString<L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn eq(&self, other: &str) -> bool {
        PartialEq::eq(self.as_str(), other)
    }
}

impl<L, UL, SM, USM, const C: usize, const UC: usize> PartialEq<ArrayString<UL, USM, UC>>
    for ArrayString<L, SM, C>
where
    L: LengthType,
    UL: LengthType,
    SM: SpareMemoryPolicy<u8>,
    USM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn eq(&self, other: &ArrayString<UL, USM, UC>) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<L, SM, const C: usize> PartialEq<alloc::string::String> for ArrayString<L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn eq(&self, other: &alloc::string::String) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl<L, SM, const C: usize> Eq for ArrayString<L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
}

#[cfg(test)]
mod testing {
    use crate as cds;
    use cds::array_str;

    #[test]
    fn test_eq_str() {
        let s = array_str![16; "cds"];
        assert_eq!(s, "cds");
        assert_eq!(s, *"cds");
        assert_eq!("cds", s);
        assert_eq!(*"cds", s);
    }

    #[test]
    fn test_eq_self() {
        let s1 = array_str![16; "cds"];
        let s2 = array_str![3; "cds"];
        assert_eq!(s1, s2);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_eq_string() {
        let string = alloc::string::String::from("cds");
        let s = array_str![8; "cds"];
        assert_eq!(s, string);
    }
}
