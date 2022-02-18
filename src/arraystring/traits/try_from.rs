use crate::{
    arraystring::ArrayString, errors::CapacityError, len::LengthType, mem::SpareMemoryPolicy,
};
use core::{convert::TryFrom, ptr, slice};

impl<L, SM, const C: usize> TryFrom<&str> for ArrayString<L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    type Error = CapacityError;

    #[inline]
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let len = s.len();
        if len > Self::CAPACITY {
            return Err(CapacityError {});
        }
        let mut tmp = Self::new_raw(len);
        unsafe {
            ptr::copy_nonoverlapping(s.as_ptr(), tmp.as_mut_ptr(), len);
            SM::init(tmp.as_mut_ptr().add(len), C - len);
        }
        Ok(tmp)
    }
}

impl<L, SM, const C: usize> TryFrom<&mut str> for ArrayString<L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    type Error = CapacityError;

    #[inline]
    fn try_from(s: &mut str) -> Result<Self, Self::Error> {
        Self::try_from(s as &str)
    }
}

impl<L, SM, const C: usize> TryFrom<char> for ArrayString<L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    type Error = CapacityError;

    #[inline]
    fn try_from(ch: char) -> Result<Self, Self::Error> {
        let ch_len = ch.len_utf8();
        if ch_len > Self::CAPACITY {
            return Err(CapacityError {});
        }
        let mut s = Self::new_raw(ch_len);
        unsafe {
            ch.encode_utf8(slice::from_raw_parts_mut(s.as_mut_ptr(), ch_len));
            SM::init(s.as_mut_ptr().add(ch_len), C - ch_len);
        }
        Ok(s)
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<L, SM, const C: usize> TryFrom<&alloc::string::String> for ArrayString<L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    type Error = CapacityError;

    #[inline]
    fn try_from(string: &String) -> Result<Self, Self::Error> {
        Self::try_from(string.as_str())
    }
}

#[cfg(test)]
mod testing {
    use crate as cds;
    use cds::{
        array_str,
        arraystring::{test_arraystring::check_spare_memory, ArrayString},
        errors::CapacityError,
        len::U8,
        mem::Pattern,
    };

    const PATTERN: u8 = 0xAB;

    #[test]
    fn test_try_from_str() {
        type S = ArrayString<U8, Pattern<PATTERN>, 255>;
        let s = S::try_from("cds").unwrap();
        assert_eq!(s, "cds");
        check_spare_memory(&s, PATTERN);
    }

    #[test]
    fn test_try_from_mut_str() {
        type S = ArrayString<U8, Pattern<PATTERN>, 255>;
        let mut src = array_str![8; "one"];
        let s = S::try_from(src.as_mut_str()).unwrap();
        assert_eq!(s, "one");
        check_spare_memory(&s, PATTERN);
    }

    #[test]
    fn test_try_from_str_err() {
        type S = ArrayString<U8, Pattern<PATTERN>, 2>;
        assert!(matches!(S::try_from("cds"), Err(CapacityError)));
    }

    #[test]
    fn test_try_from_mut_str_err() {
        type S = ArrayString<U8, Pattern<PATTERN>, 2>;
        let mut src = array_str![8; "one"];
        assert!(matches!(S::try_from(src.as_mut_str()), Err(CapacityError)));
    }

    #[test]
    fn test_try_from_char() {
        type S = ArrayString<U8, Pattern<PATTERN>, 16>;
        let s = S::try_from('a').unwrap();
        assert_eq!(s, "a");
        check_spare_memory(&s, PATTERN);
    }

    #[test]
    fn test_try_from_char_fails() {
        type S = ArrayString<U8, Pattern<PATTERN>, 2>;
        assert!(matches!(S::try_from('€'), Err(CapacityError)));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_try_from_string() {
        let string = alloc::string::String::from("cds");
        type S = ArrayString<U8, Pattern<PATTERN>, 16>;
        let s = S::try_from(&string).unwrap();
        assert_eq!(s, "cds");
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_try_from_string_fails() {
        let string = alloc::string::String::from("cds");
        type S = ArrayString<U8, Pattern<PATTERN>, 2>;
        assert!(matches!(S::try_from(&string), Err(CapacityError)));
    }
}