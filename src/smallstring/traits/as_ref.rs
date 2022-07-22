use crate::{len::LengthType, mem::SpareMemoryPolicy, smallstring::SmallString};
use core::convert::AsRef;

impl<L, SM, const C: usize> AsRef<str> for SmallString<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<L, SM, const C: usize> AsRef<[u8]> for SmallString<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl<L, SM, const C: usize> AsRef<std::ffi::OsStr> for SmallString<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn as_ref(&self) -> &std::ffi::OsStr {
        (**self).as_ref()
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl<L, SM, const C: usize> AsRef<std::path::Path> for SmallString<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn as_ref(&self) -> &std::path::Path {
        (**self).as_ref()
    }
}

#[cfg(test)]
mod testing {
    use crate as cds;
    use cds::small_str;

    #[test]
    fn test_as_ref_str() {
        let s = small_str![8; "cds"];
        let sl: &str = s.as_ref();
        assert_eq!(sl, "cds");
    }

    #[test]
    fn test_as_ref_bytes() {
        let s = small_str![8; "cds"];
        let b: &[u8] = s.as_ref();
        assert_eq!(b, b"cds");
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_as_ref_os_str() {
        let s = small_str![8; "cds"];
        let os: &std::ffi::OsStr = s.as_ref();
        assert_eq!(os, "cds");
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_as_ref_path() {
        let s = small_str![8; "cds"];
        let p: &std::path::Path = s.as_ref();
        assert_eq!(p.as_os_str(), "cds");
    }
}
