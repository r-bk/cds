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
        cds::arraystring::ArrayString::<$c>::new()
    }};
    ($c:expr; $e:expr) => {{
        cds::arraystring::ArrayString::<$c>::try_from($e)
            .expect("failed to initialize ArrayString")
    }};
}

/// Formats an [`ArrayString`] possibly truncating the result.
///
/// This macro, similar to the standard [`std::format!`], formats a string but with [`ArrayString`]
/// as the resulting type. This allows formatting a string on stack, without memory allocation.
///
/// Note that, as `ArrayString` is a fixed-capacity non-growable string,
/// the result may be truncated (on character boundary) to fit the given capacity.
///
/// This macro is a convenience wrapper of the [`format_lossy`] function.
///
/// # Examples
///
/// Format an `ArrayString` specifying the capacity only. The resulting type uses [`Usize`] as
/// length type and [`Uninitialized`] as spare memory policy.
///
/// ```rust
/// # use cds::lformat;
/// let s = lformat!(16, "Hello, world!");
/// assert_eq!(s, "Hello, world!");
/// assert_eq!(s.capacity(), 16);
/// ```
///
/// Format an `ArrayString` specifying the array-string type.
/// This allows customization of length type and spare memory policy.
///
/// ```rust
/// # use cds::{lformat, len::U8, mem::Pattern, arraystring::ArrayString};
/// type A = ArrayString<16, U8, Pattern<0xCD>>;
/// let s = lformat!(A, "Hello, world!");
/// assert_eq!(s, "Hello, world!");
/// ```
///
/// Note that the result may be truncated if `ArrayString` capacity is not enough to accommodate the
/// whole formatted string. In some use case this may yield wrong results. Thus use only where lossy
/// formatting is appropriate, or when the capacity is ensured to be enough.
///
/// ```rust
/// # use cds::lformat;
/// let s = lformat!(5, "a=2500");
/// assert_eq!(s, "a=250");  // <-- !! the result may be wrong in some use cases
/// ```
///
/// [`ArrayString`]: crate::arraystring::ArrayString
/// [`format_lossy`]: crate::arraystring::format_lossy
/// [`Usize`]: crate::len::Usize
/// [`Uninitialized`]: crate::mem::Uninitialized
#[cfg_attr(docsrs, doc(cfg(feature = "arraystring")))]
#[macro_export]
macro_rules! lformat {
    ($c:literal, $($arg:tt)*) => {{
        let res = cds::arraystring::format_lossy::<$c, cds::len::Usize, cds::mem::Uninitialized>(
            core::format_args!($($arg)*),
        );
        res
    }};
    ($s:ty, $($arg:tt)*) => {{
        let res: $s = cds::arraystring::format_lossy(core::format_args!($($arg)*));
        res
    }};
}

/// Formats an [`ArrayString`].
///
/// This macro, similar to the standard [`std::format!`], formats a string yielding
/// `Result<ArrayString>`. This allows formatting a string on stack, without memory allocation.
///
/// Note that, unlike the standard macro, this macro is fallible. `ArrayString` is a fixed-capacity
/// data structure, whose capacity may be not enough for formatting an arbitrary string.
/// See [`lformat!`] for a macro that returns a plain `ArrayString`, while possibly truncating the
/// result.
///
/// This macro is a convenience wrapper of the [`format`] function.
///
/// # Examples
///
/// Format an `ArrayString` specifying the capacity only. The resulting type uses [`Usize`] as
/// length type and [`Uninitialized`] as spare memory policy.
///
/// ```rust
/// # use cds::aformat;
/// # fn foo() -> core::fmt::Result {
/// let s = aformat!(16, "Hello, world!")?;
/// assert_eq!(s, "Hello, world!");
/// assert_eq!(s.capacity(), 16);
/// # Ok(())
/// # }
/// # foo().unwrap()
/// ```
///
/// Format an `ArrayString` specifying the array-string type.
/// This allows customization of length type and spare memory policy.
///
/// ```rust
/// # use cds::{aformat, len::U8, mem::Pattern, arraystring::ArrayString};
/// # fn foo() -> core::fmt::Result {
/// type A = ArrayString<16, U8, Pattern<0xCD>>;
/// let s = aformat!(A, "Hello, world!")?;
/// assert_eq!(s, "Hello, world!");
/// # Ok(())
/// # }
/// # foo().unwrap()
/// ```
///
/// Note that the macro may fail when there is no enough capacity.
///
/// ```rust
/// # use cds::aformat;
/// assert!(aformat!(2, "Hello, world!").is_err());
/// ```
///
/// [`ArrayString`]: crate::arraystring::ArrayString
/// [`format`]: crate::arraystring::format
/// [`Usize`]: crate::len::Usize
/// [`Uninitialized`]: crate::mem::Uninitialized
#[cfg_attr(docsrs, doc(cfg(feature = "arraystring")))]
#[macro_export]
macro_rules! aformat {
    ($c:literal, $($arg:tt)*) => {{
        cds::arraystring::format::<$c, cds::len::Usize, cds::mem::Uninitialized>(
            core::format_args!($($arg)*),
        )
    }};
    ($s:ty, $($arg:tt)*) => {{
        let res: Result<$s, core::fmt::Error> = cds::arraystring::format(core::format_args!($($arg)*));
        res
    }};
}
