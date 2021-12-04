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
