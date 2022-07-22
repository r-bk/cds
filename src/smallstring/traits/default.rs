use crate::{len::LengthType, mem::SpareMemoryPolicy, smallstring::SmallString};

impl<const C: usize, L, SM> Default for SmallString<C, L, SM>
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
    use cds::{len::U8, smallstring::SmallString};

    #[test]
    fn test_default() {
        type S = SmallString<7, U8>;
        let s: S = Default::default();
        assert_eq!(s.len(), 0);
        assert_eq!(s.capacity(), 7);
    }
}
