use crate::{
    len::LengthType, mem::errors::ReservationError, mem::SpareMemoryPolicy,
    smallstring::SmallString,
};

impl<L, SM, const C: usize> core::str::FromStr for SmallString<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    type Err = ReservationError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

#[cfg(test)]
mod testing {
    use crate as cds;
    use alloc::string::String;
    use cds::{len::U8, mem::errors::ReservationError, smallstring::SmallString};
    use core::str::FromStr;

    #[test]
    fn test_from_str() {
        type S = SmallString<4, U8>;

        let s = S::from_str("cds").unwrap();
        assert_eq!(s, "cds");

        let s = S::from_str("").unwrap();
        assert_eq!(s, "");

        let s = S::from_str("cdscds").unwrap();
        assert_eq!(s, "cdscds");

        let s = String::from_iter(['a'].iter().cycle().take(256));

        assert!(matches!(S::from_str(&s), Err(e) if e == ReservationError::CapacityOverflow));
    }
}
