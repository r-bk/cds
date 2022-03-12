use crate::{arraystring::ArrayString, len::LengthType, mem::SpareMemoryPolicy};
use core::fmt::{Debug, Formatter, Result};

impl<L, SM, const C: usize> Debug for ArrayString<C, L, SM>
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
    use crate::array_str;

    #[test]
    fn test_debug() {
        let s = array_str![16; "Hello!"];
        let f = format!("{:?}", s);
        assert_eq!(f, "\"Hello!\"");
        assert_eq!(f, format!("{:?}", String::from("Hello!")));
    }
}
