/// Creates a [`SmallString`] from the arguments.
///
/// `small_str!` macro allows creation of an `SmallString` with given capacity and content.
///
/// Note that the used length type is [`Usize`] and spare memory policy is [`Uninitialized`].
///
/// # Examples
///
/// 1. `small_str![CAPACITY;]` - create an empty `SmallString` with given local capacity.
///
/// This is equivalent to `SmallString::<CAPACITY>::new()`.
///
/// ```rust
/// # use cds::small_str;
/// let mut s = small_str![3;];
/// assert_eq!(s.len(), 0);
/// assert_eq!(s.capacity(), 3);
/// ```
///
/// 2. `small_str![CAPACITY; FROM]` - create a `SmallString` with given local capacity and from
/// initializer.
///
/// This is equivalent to `SmallString::<CAPACITY>::try_from(FROM).unwrap()`.
///
/// ```rust
/// # use cds::small_str;
/// let s = small_str![32; "Hello, world!"];
/// assert_eq!(s.capacity(), 32);
/// assert_eq!(s.len(), 13);
/// // assert_eq!(a, "Hello, world!");
/// ```
///
/// # Panics
///
/// The macro panics on memory allocation failures.ex
///
/// [`SmallString`]: crate::smallstring::SmallString
/// [`Usize`]: crate::len::Usize
/// [`Uninitialized`]: crate::mem::Uninitialized
#[cfg_attr(docsrs, doc(cfg(feature = "smallstring")))]
#[macro_export]
macro_rules! small_str {
    ($c:expr;) => {{
        cds::smallstring::SmallString::<$c>::new()
    }};
    ($c:expr; $e:expr) => {{
        cds::smallstring::SmallString::<$c>::try_from($e).expect("small_str! failed")
    }};
}
