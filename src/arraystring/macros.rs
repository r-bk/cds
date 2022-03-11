/// Creates an [`ArrayString`] containing the arguments.
///
/// `array_str!` macro allows creation of an `ArrayString` with given capacity and content.
///
/// Note that the used length type is [`Usize`] and spare memory policy is [`Uninitialized`].
///
/// # Examples
///
/// 1. `array_str![CAPACITY;]` - create an empty `ArrayString` with given capacity:
///
/// ```rust
/// # use cds::array_str;
/// let mut a = array_str![3;];
/// assert_eq!(a.len(), 0);
/// assert_eq!(a.capacity(), 3);
/// ```
///
/// 4. `array_str![CAPACITY; TRY_FROM]` - create an `ArrayString` with given capacity and
/// initializer:
///
/// ```rust
/// # use cds::array_str;
/// let a = array_str![32; "Hello, world!"];
/// assert_eq!(a.capacity(), 32);
/// assert_eq!(a.len(), 13);
/// // assert_eq!(a, "Hello, world!");
/// ```
///
/// 5. `array_str!` panics if the number of elements exceeds the requested capacity:
///
/// ```should_panic
/// # use cds::array_str;
/// array_str![0; "Hello, world!"];
/// ```
///
/// # Panics
///
/// The macro panics if initialization of the array-string fails.
///
/// [`ArrayString`]: crate::arraystring::ArrayString
/// [`Usize`]: crate::len::Usize
/// [`Uninitialized`]: crate::mem::Uninitialized
#[cfg_attr(docsrs, doc(cfg(feature = "arraystring")))]
#[macro_export]
macro_rules! array_str {
    ($c:expr;) => {{
        cds::arraystring::ArrayString::<cds::len::Usize, cds::mem::Uninitialized, $c>::new()
    }};
    ($c:expr; $e:expr) => {{
        cds::arraystring::ArrayString::<cds::len::Usize, cds::mem::Uninitialized, $c>::try_from($e)
            .expect("failed to initialize ArrayString")
    }};
}

/// Formats an [`ArrayString`] with the arguments.
///
/// This macro, similar to the standard [`std::format!`], formats a string but with [`ArrayString`]
/// as the resulting type. This allows formatting a string on stack, without memory allocation.
///
/// Note that, as `ArrayString` is a fixed-capacity non-growable string,
/// the result may be truncated (on character boundary) to fit the given capacity.
///
/// This macro is a convenience wrapper of the [`format`] function.
///
/// # Examples
///
/// Format an `ArrayString` specifying the capacity only. The resulting type uses [`Usize`] as
/// length type and [`Uninitialized`] as spare memory policy.
///
/// ```rust
/// # use cds::format;
/// let s = format!(16, "Hello, world!");
/// assert_eq!(s, "Hello, world!");
/// assert_eq!(s.capacity(), 16);
/// ```
///
/// Format an `ArrayString` specifying the whole type. This allows customization of the length type
/// and spare memory policy.
///
/// ```rust
/// # use cds::{format, len::U8, mem::Pattern, arraystring::ArrayString};
/// type A = ArrayString<U8, Pattern<0xCD>, 16>;
/// let s = format!(A, "Hello, world!");
/// assert_eq!(s, "Hello, world!");
/// ```
///
/// The result may be truncated if `ArrayString` capacity is not enough to accommodate the whole
/// formatted string.
///
/// ```rust
/// # use cds::format;
/// let s = format!(5, "100€");  // <-- '€' in UTF-8 is 3 bytes long
/// assert_eq!(s, "100");        // <-- the result is truncated on character boundary
/// ```
///
/// [`std::format`]: std::format
/// [`ArrayString`]: crate::arraystring::ArrayString
/// [`format`]: crate::arraystring::format
/// [`format!`]: crate::format
/// [`Usize`]: crate::len::Usize
/// [`Uninitialized`]: crate::mem::Uninitialized
#[cfg_attr(docsrs, doc(cfg(feature = "arraystring")))]
#[macro_export]
macro_rules! format {
    ($c:literal, $($arg:tt)*) => {{
        let res = cds::arraystring::format::<cds::len::Usize, cds::mem::Uninitialized, $c>(
            core::format_args!($($arg)*),
        );
        res
    }};
    ($s:ty, $($arg:tt)*) => {{
        let res: $s = cds::arraystring::format(core::format_args!($($arg)*));
        res
    }};
}
