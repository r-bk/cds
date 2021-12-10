use crate::{
    arrayvec::ArrayVec,
    defs::{LengthType, SpareMemoryPolicy},
};
use core::fmt::{Debug, Formatter, Result};

impl<T, L, SM, const C: usize> Debug for ArrayVec<T, L, SM, C>
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
    use crate::array_vec;

    #[test]
    fn test_debug() {
        let mut a = array_vec![3; String];
        a.push("Hello".into());
        a.push(", ".into());
        a.push("world!".into());
        let s = format!("{:?}", a);
        assert_eq!(s, "[\"Hello\", \", \", \"world!\"]");
    }
}
