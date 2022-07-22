use crate::{len::LengthType, mem::SpareMemoryPolicy, smallstring::SmallString};
use core::fmt::{Display, Formatter, Result};

impl<L, SM, const C: usize> Display for SmallString<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Display::fmt(&**self, f)
    }
}

#[cfg(all(test, feature = "std"))]
mod testing {
    use crate as cds;
    use cds::small_str;

    #[test]
    fn test_display() {
        let s = small_str![8; "cds"];
        assert_eq!(format!("{}", s), String::from("cds"));
    }
}
