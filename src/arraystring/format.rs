use crate::{arraystring::ArrayString, len::LengthType, mem::SpareMemoryPolicy};
use core::fmt::{Arguments, Write};

struct PartialWriter<'a, L: LengthType, SM: SpareMemoryPolicy<u8>, const C: usize>(
    &'a mut ArrayString<L, SM, C>,
);

impl<'a, L, SM, const C: usize> Write for PartialWriter<'a, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.0.add_str(s);
        Ok(())
    }

    #[inline]
    fn write_char(&mut self, c: char) -> core::fmt::Result {
        self.0.try_push(c).ok();
        Ok(())
    }
}

/// Formats an `ArrayString` from arguments.
///
/// This function allows formatting a string, similar to the standard [`format`] function,
/// but with `ArrayString` as the resulting type. This allows formatting a string on stack,
/// without memory allocation.
///
/// The [`Arguments`] instance can be created with the [`format_args!`] macro.
///
/// Note that, as `ArrayString` is a fixed-capacity non-growable string,
/// the result may be truncated (on character boundary) to fit the given capacity.
///
/// # Examples
///
/// ```rust
/// # use cds::{arraystring::{format, ArrayString}, len::U8, mem::Uninitialized};
/// # use core::format_args;
/// type S = ArrayString<U8, Uninitialized, 16>;
/// let s: S = format(format_args!("Hello, world!"));
/// assert_eq!(s, "Hello, world!");
/// ```
///
/// The result may be silently truncated if there is no enough capacity:
/// ```rust
/// # use cds::{arraystring::{format, ArrayString}, len::U8, mem::Uninitialized};
/// # use core::format_args;
/// type S = ArrayString<U8, Uninitialized, 4>;  // <-- not enough capacity
/// let s: S = format(format_args!("25€"));
/// assert_eq!(s, "25");  // <-- the result is truncated on character boundary
/// ```
///
/// [`format`]: std::fmt::format
/// [`format_args!`]: core::format_args
#[inline]
pub fn format<L, SM, const C: usize>(args: Arguments<'_>) -> ArrayString<L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    let mut s = ArrayString::<L, SM, C>::new();
    let mut pw = PartialWriter(&mut s);
    pw.write_fmt(args).ok();
    s
}

#[cfg(test)]
mod testing {
    use crate as cds;

    #[test]
    fn test_format() {
        let s = cds::format!(16, "Hello, world!");
        assert_eq!(s, "Hello, world!");

        let s = cds::format!(16, "{}", 'A');
        assert_eq!(s, "A");

        let s = cds::format!(5, "2€€");
        assert_eq!(s, "2€");

        let s = cds::format!(0, "cds");
        assert_eq!(s, "");
    }
}
