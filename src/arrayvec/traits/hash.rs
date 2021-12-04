use crate::{arrayvec::ArrayVec, defs::SpareMemoryPolicy};
use core::hash::{Hash, Hasher};

impl<T, SM, const C: usize> Hash for ArrayVec<T, SM, C>
where
    T: Hash,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state)
    }
}
