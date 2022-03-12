use crate::{len::LengthType, mem::SpareMemoryPolicy, smallvec::SmallVec};
use core::fmt::{Debug, Formatter, Result};

impl<T, L, SM, const C: usize> Debug for SmallVec<T, C, L, SM>
where
    T: Debug,
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
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
    use crate::small_vec;

    #[test]
    fn test_debug() {
        let mut v = small_vec![3; String];
        v.push("Hello".into());
        v.push(", ".into());
        v.push("world!".into());
        let s = format!("{:?}", v);
        assert_eq!(s, "[\"Hello\", \", \", \"world!\"]");
    }
}
