use crate::{arrayvec::ArrayVec, mem::SpareMemoryPolicy};
use core::default::Default;

impl<T, SM, const C: usize> Default for ArrayVec<T, SM, C>
where
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
