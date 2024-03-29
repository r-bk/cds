use crate::{arraystring::ArrayString, len::LengthType, mem::SpareMemoryPolicy};
use core::fmt::{self, Arguments, Write};

struct LossyWriter<'a, const C: usize, L: LengthType, SM: SpareMemoryPolicy<u8>>(
    &'a mut ArrayString<C, L, SM>,
);

impl<'a, L, SM, const C: usize> Write for LossyWriter<'a, C, L, SM>
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

/// Formats an `ArrayString` possibly truncating the result.
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
/// # use cds::{arraystring::{format_lossy, ArrayString}, len::U8};
/// # use core::format_args;
/// type S = ArrayString<16, U8>;
/// let s: S = format_lossy(format_args!("Hello, world!"));
/// assert_eq!(s, "Hello, world!");
/// ```
///
/// The result may be silently truncated if there is no enough capacity. Use only when lossy
/// formatting is appropriate, or when the capacity is ensured to be enough.
/// ```rust
/// # use cds::{arraystring::{format_lossy, ArrayString}, len::U8};
/// # use core::format_args;
/// type S = ArrayString<4, U8>;  // <-- not enough capacity
/// let s: S = format_lossy(format_args!("25€"));
/// assert_eq!(s, "25");  // <-- the result is truncated on character boundary
///
/// let s: S = format_lossy(format_args!("a=2500"));
/// assert_eq!(s, "a=25");  // <-- !! the result may be completely wrong in some use cases
/// ```
///
/// [`format`]: std::fmt::format
/// [`format_args!`]: core::format_args
#[inline]
pub fn format_lossy<const C: usize, L, SM>(args: Arguments<'_>) -> ArrayString<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    let mut s = ArrayString::<C, L, SM>::new();
    let mut pw = LossyWriter(&mut s);
    pw.write_fmt(args).ok();
    s
}

/// Formats an `ArrayString`.
///
/// This function allows formatting an `ArrayString` similar to the standard [`format`] function.
/// However, as `ArrayString` is a non-growable string, formatting it may fail due to lack of
/// capacity. Thus, unlike the standard function, this function returns `Result<ArrayString>`
/// instead. See [`format_lossy`] for a function that returns a plain `ArrayString`, possibly
/// truncating the result when capacity is insufficient.
///
/// The [`Arguments`] instance can be created with the [`format_args!`] macro.
/// See the [`aformat!`] macro for a convenience wrapper of this function.
///
/// # Examples
///
/// ```rust
/// # use cds::{arraystring::{format, ArrayString}, len::U8};
/// # use core::format_args;
/// # fn foo() -> core::fmt::Result{
/// type S = ArrayString<16, U8>;
/// let s: S = format(format_args!("Hello, world!"))?;
/// assert_eq!(s, "Hello, world!");
/// # Ok(())
/// # }
/// # foo().unwrap();
/// ```
/// Note that the function may fail when there is no enough capacity in `ArrayString`.
///
/// ```rust
/// # use cds::{arraystring::{format, ArrayString}, len::U8};
/// # use core::{fmt, format_args};
/// type S = ArrayString<5, U8>;
/// let res: Result<S, fmt::Error> = format(format_args!("Hello, world!"));
/// assert!(res.is_err());
/// ```
///
/// [`format`]: std::fmt::format
/// [`aformat!`]: crate::aformat
#[inline]
pub fn format<const C: usize, L, SM>(
    args: Arguments<'_>,
) -> Result<ArrayString<C, L, SM>, fmt::Error>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    let mut s = ArrayString::<C, L, SM>::new();
    s.write_fmt(args)?;
    Ok(s)
}

#[cfg(test)]
mod testing {
    use crate as cds;

    #[test]
    fn test_format_lossy() {
        let s = cds::lformat!(16, "Hello, world!");
        assert_eq!(s, "Hello, world!");

        let s = cds::lformat!(16, "{}", 'A');
        assert_eq!(s, "A");

        let s = cds::lformat!(5, "2€€");
        assert_eq!(s, "2€");

        let s = cds::lformat!(0, "cds");
        assert_eq!(s, "");
    }
}
