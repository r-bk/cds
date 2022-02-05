/// Creates an [`ArrayVec`] containing the arguments.
///
/// `array_vec!` macro allows creation of an `ArrayVec` using syntax similar to that of the standard
/// array.
///
/// # Examples
///
/// 1. `array_vec![CAPACITY; TYPE]` - create an empty `ArrayVec` with given capacity and type:
///
/// ```rust
/// # use cds::array_vec;
/// let a = array_vec![3; u64];
/// assert_eq!(a.capacity(), 3);
/// assert_eq!(a.len(), 0);
/// ```
///
/// 2. `array_vec![CAPACITY; TYPE; ELEM+]` - create an `ArrayVec` with given capacity, type and
/// elements:
///
/// ```rust
/// # use cds::array_vec;
/// let a = array_vec![5; u64; 17];
/// assert_eq!(a.capacity(), 5);
/// assert_eq!(a.len(), 1);
/// assert_eq!(a[0], 17u64);
/// ```
///
/// 3. `array_vec![CAPACITY;]` - create an empty `ArrayVec` with given capacity, let the compiler
/// derive the type:
///
/// ```rust
/// # use cds::array_vec;
/// let mut a = array_vec![3;];
/// a.push("str");
/// assert_eq!(a.capacity(), 3);
/// assert_eq!(a.len(), 1);
/// assert_eq!(a[0], "str");
/// ```
///
/// 4. `array_vec![CAPACITY; ELEM+]` - create an `ArrayVec` with given capacity and elements,
/// let the compiler derive the type:
///
/// ```rust
/// # use cds::array_vec;
/// let a = array_vec![32; 9, 8, 7];
/// assert_eq!(a.capacity(), 32);
/// assert_eq!(a.len(), 3);
/// assert_eq!(&a[..], &[9, 8, 7]);
/// ```
///
/// 5. `array_vec!` panics if the number of elements exceeds the requested capacity:
///
/// ```should_panic
/// # use cds::array_vec;
/// array_vec![0; u64; 1];
/// ```
///
/// # Panics
///
/// The macro panics if the number of elements exceeds the requested capacity.
///
/// [`ArrayVec`]: crate::arrayvec::ArrayVec
#[cfg_attr(docsrs, doc(cfg(feature = "arrayvec")))]
#[macro_export]
macro_rules! array_vec {
    ($c:expr; $t:ty) => {{
        cds::arrayvec::ArrayVec::<$t, cds::defs::Usize, cds::defs::Uninitialized, $c>::new()
    }};
    ($c:expr; $t:ty; $($e:expr),+ $(,)?) => {{
        cds::arrayvec::ArrayVec::<$t, cds::defs::Usize, cds::defs::Uninitialized, $c>::try_from([$($e),*])
            .expect("insufficient capacity")
    }};
    ($c:expr;) => {{
        cds::arrayvec::ArrayVec::<_, cds::defs::Usize, cds::defs::Uninitialized, $c>::new()
    }};
    ($c:expr; $($e:expr),+ $(,)?) => {{
        cds::arrayvec::ArrayVec::<_, cds::defs::Usize, cds::defs::Uninitialized, $c>::try_from([$($e),*])
            .expect("insufficient capacity")
    }};
}
