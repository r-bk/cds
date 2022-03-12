use crate::{arraystring::ArrayString, len::LengthType, mem::SpareMemoryPolicy};
use core::fmt::{Error, Result, Write};

/// Implementation of [`Write`] for [`ArrayString`].
///
/// Note that, as `ArrayString` is a fixed-capacity non-growable writer,
/// these methods may fail due to capacity constraints.
///
/// See [`lformat!`] for lossy formatting.
///
/// [`lformat!`]: crate::lformat
impl<L, SM, const C: usize> Write for ArrayString<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn write_str(&mut self, s: &str) -> Result {
        self.try_push_str(s).map_err(|_| Error {})
    }

    #[inline]
    fn write_char(&mut self, c: char) -> Result {
        self.try_push(c).map_err(|_| Error {})
    }
}

#[cfg(test)]
mod testing {
    use crate as cds;
    use cds::array_str;
    use core::fmt::Write;

    #[test]
    fn test_write_str() {
        let mut s = array_str![16;];
        assert!(core::write!(&mut s, "Hello, world!").is_ok());
        assert_eq!(s, "Hello, world!");
    }

    #[test]
    fn test_write_str_fails() {
        let mut s = array_str![7;];
        assert!(matches!(
            core::write!(&mut s, "Hello, {}!", "world"),
            Err(core::fmt::Error)
        ));
        assert_eq!(s, "Hello, ");
    }

    #[test]
    fn test_write_char() {
        let mut s = array_str![1;];
        assert!(core::write!(&mut s, "{}", 'A').is_ok());
        assert_eq!(s, "A");
    }

    #[test]
    fn test_write_char_fails() {
        let mut s = array_str![1;];
        assert!(matches!(
            core::write!(&mut s, "{}", '€'),
            Err(core::fmt::Error)
        ));
    }
}
