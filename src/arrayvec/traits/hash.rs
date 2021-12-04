use crate::{
    arrayvec::ArrayVec,
    defs::{LengthType, SpareMemoryPolicy},
};
use core::hash::{Hash, Hasher};

impl<T, L, SM, const C: usize> Hash for ArrayVec<T, L, SM, C>
where
    T: Hash,
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state)
    }
}
