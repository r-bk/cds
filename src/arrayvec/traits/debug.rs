use crate::ArrayVec;
use core::fmt::{Debug, Formatter, Result};

impl<T, const C: usize> Debug for ArrayVec<T, C>
where
    T: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Debug::fmt(&**self, f)
    }
}
