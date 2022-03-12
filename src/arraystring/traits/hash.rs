use crate::{arraystring::ArrayString, len::LengthType, mem::SpareMemoryPolicy};
use core::hash::{Hash, Hasher};

impl<L, SM, const C: usize> Hash for ArrayString<C, L, SM>
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
    use cds::array_str;
    use core::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn test_hash() {
        let mut hasher1 = DefaultHasher::new();
        let a = array_str![3; "cds"];
        a.hash(&mut hasher1);

        let mut hasher2 = DefaultHasher::new();
        "cds".hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }
}
