use crate::{
    arraystring::ArrayString, errors::CapacityError, len::LengthType, mem::SpareMemoryPolicy,
};

impl<L, SM, const C: usize> core::str::FromStr for ArrayString<L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    type Err = CapacityError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

#[cfg(test)]
mod testing {
    use crate as cds;
    use cds::{arraystring::ArrayString, errors::CapacityError, len::U8, mem::Uninitialized};
    use core::str::FromStr;

    #[test]
    fn test_from_str() {
        type AS = ArrayString<U8, Uninitialized, 4>;

        let s = AS::from_str("cds").unwrap();
        assert_eq!(s, "cds");

        let s = AS::from_str("").unwrap();
        assert_eq!(s, "");

        assert!(matches!(AS::from_str("abcdef"), Err(CapacityError)));
    }
}
