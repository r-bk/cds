/// Creates a [`SmallVec`] containing the arguments.
///
/// `small_vec!` macro allows creation of a `SmallVec` using syntax similar to that of the standard
/// array.
///
/// # Examples
///
/// 1. `small_vec![CAPACITY; TYPE]` - create an empty `SmallVec` with given local capacity and
/// element type:
///
/// ```rust
/// # use cds::small_vec;
/// let v = small_vec![3; u64];
/// assert_eq!(v.capacity(), 3);
/// assert_eq!(v.is_empty(), true);
/// assert_eq!(v.is_heap(), false);
/// ```
///
/// 2. `small_vec![CAPACITY; TYPE; ELEM+]` - create a `SmallVec` with given local capacity,
/// element type and element values:
///
/// ```rust
/// # use cds::small_vec;
/// let v = small_vec![5; u64; 17];
/// assert_eq!(v.capacity(), 5);
/// assert_eq!(v.len(), 1);
/// assert_eq!(v[0], 17u64);
/// ```
///
/// 3. `small_vec![CAPACITY;]` - create an empty `SmallVec` with given local capacity,
/// let the compiler derive the element type:
///
/// ```rust
/// # use cds::small_vec;
/// let mut v = small_vec![3;];
/// v.push("str");
/// assert_eq!(v.capacity(), 3);
/// assert_eq!(v.len(), 1);
/// assert_eq!(v[0], "str");
/// ```
///
/// 4. `small_vec![CAPACITY; ELEM+]` - create a `SmallVec` with given local capacity and elements,
/// let the compiler derive the element type:
///
/// ```rust
/// # use cds::small_vec;
/// let v = small_vec![32; 9, 8, 7];
/// assert_eq!(v.capacity(), 32);
/// assert_eq!(v.len(), 3);
/// assert_eq!(&v[..], &[9, 8, 7]);
/// ```
///
/// # Panics
///
/// The macro panics if non-empty small-vector is created and dynamic memory allocation fails.
/// See [`SmallVec::reserve_exact`] for more information.
///
/// [`SmallVec`]: crate::smallvec::SmallVec
/// [`SmallVec::reserve_exact`]: crate::smallvec::SmallVec::reserve_exact
#[cfg_attr(docsrs, doc(cfg(feature = "smallvec")))]
#[macro_export]
macro_rules! small_vec {
    ($c:expr; $t:ty) => {{
        cds::smallvec::SmallVec::<$t, $c>::new()
    }};
    ($c:expr; $t:ty; $($e:expr),+ $(,)?) => {{
        cds::smallvec::SmallVec::<$t, $c>::try_from([$($e),*])
            .expect("small_vec! failed")
    }};
    ($c:expr;) => {{
        cds::smallvec::SmallVec::<_, $c>::new()
    }};
    ($c:expr; $($e:expr),+ $(,)?) => {{
        cds::smallvec::SmallVec::<_, $c>::try_from([$($e),*])
            .expect("small_vec! failed")
    }};
}
