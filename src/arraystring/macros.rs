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
