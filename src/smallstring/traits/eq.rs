use crate::{len::LengthType, mem::SpareMemoryPolicy, smallstring::SmallString};
use core::cmp::{Eq, PartialEq};

impl<'a, const C: usize, L, SM> PartialEq<&'a str> for SmallString<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        PartialEq::eq(self.as_str(), *other)
    }
}

impl<'a, const C: usize, L, SM> PartialEq<SmallString<C, L, SM>> for &'a str
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn eq(&self, other: &SmallString<C, L, SM>) -> bool {
        PartialEq::eq(*self, other.as_str())
    }
}

impl<const C: usize, L, SM> PartialEq<SmallString<C, L, SM>> for str
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn eq(&self, other: &SmallString<C, L, SM>) -> bool {
        PartialEq::eq(self, other.as_str())
    }
}

impl<const C: usize, L, SM> PartialEq<str> for SmallString<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn eq(&self, other: &str) -> bool {
        PartialEq::eq(self.as_str(), other)
    }
}

impl<const C: usize, const UC: usize, L, UL, SM, USM> PartialEq<SmallString<UC, UL, USM>>
    for SmallString<C, L, SM>
where
    L: LengthType,
    UL: LengthType,
    SM: SpareMemoryPolicy<u8>,
    USM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn eq(&self, other: &SmallString<UC, UL, USM>) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl<const C: usize, L, SM> PartialEq<alloc::string::String> for SmallString<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn eq(&self, other: &alloc::string::String) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl<L, SM, const C: usize> Eq for SmallString<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
}

#[cfg(test)]
mod testing {
    use crate as cds;
    use cds::small_str;

    #[test]
    fn test_eq_str() {
        let s = small_str![16; "cds"];
        assert_eq!(s, "cds");
        assert_eq!(s, *"cds");
        assert_eq!("cds", s);
        assert_eq!(*"cds", s);
    }

    #[test]
    fn test_eq_self() {
        let s1 = small_str![16; "cds"];
        let s2 = small_str![3; "cds"];
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_eq_string() {
        let string = alloc::string::String::from("cds");
        let s = small_str![8; "cds"];
        assert_eq!(s, string);
    }
}
