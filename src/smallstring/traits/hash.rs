use crate::{len::LengthType, mem::SpareMemoryPolicy, smallstring::SmallString};
use core::hash::{Hash, Hasher};

impl<L, SM, const C: usize> Hash for SmallString<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state)
    }
}

#[cfg(all(test, feature = "std"))]
mod testing {
    use crate as cds;
    use cds::small_str;
    use core::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn test_hash() {
        let mut hasher1 = DefaultHasher::new();
        let s = small_str![3; "cds"];
        s.hash(&mut hasher1);

        let mut hasher2 = DefaultHasher::new();
        "cds".hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }
}
