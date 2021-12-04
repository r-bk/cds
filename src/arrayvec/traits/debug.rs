use crate::{arrayvec::ArrayVec, defs::SpareMemoryPolicy};
use core::fmt::{Debug, Formatter, Result};

impl<T, SM, const C: usize> Debug for ArrayVec<T, SM, C>
where
    T: Debug,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Debug::fmt(&**self, f)
    }
}
