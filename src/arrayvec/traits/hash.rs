use crate::{arrayvec::ArrayVec, defs::LengthType, mem::SpareMemoryPolicy};
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

#[cfg(all(test, feature = "std"))]
mod testing {
    use crate as cds;
    use cds::array_vec;
    use core::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn test_hash() {
        let mut hasher1 = DefaultHasher::new();
        let a = array_vec![3; u64; 3, 1, 2];
        a.hash(&mut hasher1);

        let mut hasher2 = DefaultHasher::new();
        let arr: [u64; 3] = [3, 1, 2];
        arr.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }
}
