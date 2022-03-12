use crate::{arraystring::ArrayString, len::LengthType, mem::SpareMemoryPolicy};
use core::default::Default;

impl<L, SM, const C: usize> Default for ArrayString<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod testing {
    use crate as cds;
    use cds::{arraystring::ArrayString, len::U8};

    #[test]
    fn test_default() {
        type A = ArrayString<7, U8>;
        let a: A = Default::default();
        assert_eq!(a.len(), 0);
        assert_eq!(a.capacity(), 7);
        assert_eq!(a.spare_capacity(), a.capacity());
    }
}
