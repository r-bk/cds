use crate::{len::LengthType, mem::SpareMemoryPolicy, smallvec::SmallVec};
use core::default::Default;

impl<T, const C: usize, L, SM> Default for SmallVec<T, C, L, SM>
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
    use cds::smallvec::SmallVec;

    #[test]
    fn test_default() {
        let sv = SmallVec::<u64, 2>::default();
        assert!(sv.is_empty());
        assert_eq!(sv.capacity(), 2);
    }
}
