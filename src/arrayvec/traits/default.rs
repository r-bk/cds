use crate::{
    arrayvec::ArrayVec,
    defs::{LengthType, SpareMemoryPolicy},
};
use core::default::Default;

impl<T, L, SM, const C: usize> Default for ArrayVec<T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod testing {
    use crate as cds;
    use cds::{
        arrayvec::ArrayVec,
        defs::{Uninitialized, U8},
    };

    #[test]
    fn test_default() {
        type A = ArrayVec<u8, U8, Uninitialized, 7>;
        let a: A = Default::default();
        assert_eq!(a.len(), 0);
        assert_eq!(a.capacity(), 7);
        assert_eq!(a.spare_capacity_len(), a.capacity());
    }
}
