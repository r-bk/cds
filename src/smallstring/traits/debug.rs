use crate::{len::LengthType, mem::SpareMemoryPolicy, smallstring::SmallString};
use core::fmt::{Debug, Formatter, Result};

impl<const C: usize, L, SM> Debug for SmallString<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Debug::fmt(&**self, f)
    }
}

#[cfg(feature = "std")]
#[cfg(test)]
mod testing {
    use crate as cds;
    use crate::small_str;

    #[test]
    fn test_debug() {
        let s = small_str![3; "cds"];
        let tmp = format!("{:?}", s);
        assert_eq!(tmp, "\"cds\"");
    }
}
