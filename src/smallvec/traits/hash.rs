use crate::{len::LengthType, mem::SpareMemoryPolicy, smallvec::SmallVec};
use core::hash::{Hash, Hasher};

impl<T, L, SM, const C: usize> Hash for SmallVec<T, C, L, SM>
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

#[cfg(all(test, feature = "std"))]
mod testing {
    use crate as cds;
    use cds::small_vec;
    use core::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn test_hash() {
        let mut hasher1 = DefaultHasher::new();
        let v = small_vec![3; u64; 3, 1, 2];
        v.hash(&mut hasher1);

        let mut hasher2 = DefaultHasher::new();
        let arr: [u64; 3] = [3, 1, 2];
        arr.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }
}
