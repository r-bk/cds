use crate::{
    len::LengthType,
    mem::{errors::ReservationError, SpareMemoryPolicy},
    smallstring::SmallString,
};
use core::{mem::MaybeUninit, ptr};

impl<const C: usize, L, SM> TryFrom<&str> for SmallString<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    type Error = ReservationError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        unsafe { Self::try_from_bytes(value.as_bytes()) }
    }
}

impl<const C: usize, L, SM> TryFrom<&mut str> for SmallString<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    type Error = ReservationError;

    #[inline]
    fn try_from(s: &mut str) -> Result<Self, Self::Error> {
        unsafe { Self::try_from_bytes(s.as_bytes()) }
    }
}

impl<const C: usize, L, SM> TryFrom<&alloc::string::String> for SmallString<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    type Error = ReservationError;

    #[inline]
    fn try_from(string: &alloc::string::String) -> Result<Self, Self::Error> {
        Self::try_from(string.as_str())
    }
}

impl<const C: usize, L, SM> TryFrom<char> for SmallString<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    type Error = ReservationError;

    #[inline]
    fn try_from(ch: char) -> Result<Self, Self::Error> {
        unsafe {
            let mut buf: [MaybeUninit<u8>; 8] = MaybeUninit::uninit().assume_init();
            let slice = ptr::slice_from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, buf.len());
            let s = ch.encode_utf8(&mut *slice);
            Self::try_from_bytes(s.as_bytes())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate as cds;
    use alloc::string::String;
    use cds::smallstring::SmallString;

    type S2 = SmallString<2>;
    type S4 = SmallString<4>;

    #[test]
    fn test_try_from_str() {
        for v in ["", "cds", "toto", "hello, world!"] {
            let s = S4::try_from(v).unwrap();
            assert_eq!(s, v);
        }
    }

    #[test]
    fn test_try_from_mut_str() {
        for v in ["", "cds", "toto", "hello, world!"] {
            let mut value: String = v.into();
            let s = S4::try_from(value.as_mut_str()).unwrap();
            assert_eq!(s, v);
        }
    }

    #[test]
    fn test_try_from_string() {
        for v in ["", "cds", "toto", "hello, world!"] {
            let value: String = v.into();
            let s = S4::try_from(&value).unwrap();
            assert_eq!(s, v);
        }
    }

    #[test]
    fn test_try_from_char() {
        let mut buf: [u8; 4] = [0; 4];

        for (idx, c) in ['\u{0041}', '\u{00C5}', '\u{9860}', '\u{200D0}']
            .iter()
            .enumerate()
        {
            assert_eq!(c.len_utf8(), idx + 1);

            let s = S2::try_from(*c).unwrap();
            assert_eq!(s, String::try_from(*c).unwrap());

            let sb = c.encode_utf8(&mut buf[..]);
            assert_eq!(s.as_str(), sb);
        }
    }
}
