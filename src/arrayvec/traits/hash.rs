use crate::arrayvec::ArrayVec;
use core::hash::{Hash, Hasher};

impl<T, const C: usize> Hash for ArrayVec<T, C>
where
    T: Hash,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state)
    }
}
