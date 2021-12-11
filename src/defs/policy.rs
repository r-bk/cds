use core::ptr;

mod private {
    pub trait SpareMemoryPolicyBase<T> {
        unsafe fn init(dst: *mut T, count: usize);
    }
}

/// A trait of custom spare memory policies.
///
/// A spare memory policy defines the way a *cds* collection handles spare memory.
///
/// This includes both the initial allocation, and any memory that becomes free following removal of
/// an element from a collection (including removal forced by a [`drop`] of a collection).
///
/// Note that spare memory policy affects only the current region of memory occupied by a
/// collection. That is, if a program moves a whole collection (e.g. `ArrayVec`), the old region of
/// memory occupied by the collection is no longer accessible, and spare memory policy cannot be
/// applied to it. This may lead to having the old region of memory a bytewise copy of
/// the memory the collection was moved to.
///
/// Currently the following policies are supported:
///
/// - [`Uninitialized`] does nothing with spare bytes
/// - [`Zeroed`] fills spare bytes with zeroes
/// - [`Pattern`] fills spare bytes with a specified value
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "arrayvec")]
/// use cds::{
///     arrayvec::ArrayVec,
///     defs::{Uninitialized, Pattern, Zeroed, U8}
/// };
/// use core::convert::TryFrom;
/// # #[cfg(feature = "arrayvec")]
/// # fn example() -> Result<(), cds::errors::CapacityError> {
///
/// // --- Uninitialized ---
///
/// type A = ArrayVec<u16, U8, Uninitialized, 3>;
///
/// // all memory is uninitialized
/// let mut a = A::new();
///
/// // all memory except for the first element is uninitialized
/// let mut a = A::try_from([1])?;
/// assert_eq!(unsafe { a.as_ptr().read() }, 1);
///
/// // spare memory remains a bytewise copy of elements previously stored there
/// assert_eq!(a.pop(), Some(1));
/// assert_eq!(unsafe { a.as_ptr().read() }, 1);
///
///
/// // --- Pattern ---
///
/// type B = ArrayVec<u16, U8, Pattern<0xAB>, 3>;
///
/// // all memory is initialized with P
/// let mut a = B::new();
/// assert_eq!(unsafe { a.as_ptr().add(0).read() }, 0xABAB);
/// assert_eq!(unsafe { a.as_ptr().add(1).read() }, 0xABAB);
/// assert_eq!(unsafe { a.as_ptr().add(2).read() }, 0xABAB);
///
/// // all memory except for the first two elements is initialized with P
/// let mut a = B::try_from([1, 2])?;
/// assert_eq!(unsafe { a.as_ptr().add(0).read() }, 0x1);
/// assert_eq!(unsafe { a.as_ptr().add(1).read() }, 0x2);
/// assert_eq!(unsafe { a.as_ptr().add(2).read() }, 0xABAB);
///
/// // spare memory is initialized with P after removal of an element
/// assert_eq!(a.remove(0), 1);
/// assert_eq!(unsafe { a.as_ptr().add(0).read() }, 0x2);
/// assert_eq!(unsafe { a.as_ptr().add(1).read() }, 0xABAB);
/// assert_eq!(unsafe { a.as_ptr().add(2).read() }, 0xABAB);
///
///
/// // --- Zeroed ---
///
/// type C = ArrayVec<u16, U8, Zeroed, 2>;
///
/// // all memory is zeroed
/// let a = C::new();
/// assert_eq!(unsafe { a.as_ptr().add(0).read() }, 0);
/// assert_eq!(unsafe { a.as_ptr().add(1).read() }, 0);
///
/// // all memory except for the first element is zeroed
/// let mut a = C::try_from([3])?;
/// assert_eq!(unsafe { a.as_ptr().add(0).read() }, 3);
/// assert_eq!(unsafe { a.as_ptr().add(1).read() }, 0);
///
/// // spare memory is zeroed after removal of an element
/// assert_eq!(a.pop(), Some(3));
/// assert_eq!(unsafe { a.as_ptr().add(0).read() }, 0);
/// assert_eq!(unsafe { a.as_ptr().add(1).read() }, 0);
/// # Ok(())
/// # }
/// # #[cfg(feature = "arrayvec")]
/// # example().expect("example failed");
/// ```
///
/// [`drop`]: https://doc.rust-lang.org/core/mem/fn.drop.html
pub trait SpareMemoryPolicy<T>: private::SpareMemoryPolicyBase<T> {}

/// Uninitialized spare memory policy.
///
/// `Uninitialized` is the fastest spare memory policy, because it does absolutely nothing.
///
/// This means that:
/// - initial memory allocated by a collection remains uninitialized
/// - a region of memory occupied by an element remains a bytewise copy of the
///   element after it is moved out, until the region is overwritten (if at all)
/// - a region of memory occupied by a collection remains untouched when the collection is dropped,
///   until the region is overwritten (if at all)
pub struct Uninitialized;

/// Pattern-initialized spare memory policy.
///
/// Written as `Pattern<P>`, pattern spare memory policy initializes every spare byte with the value
/// `P`.
///
/// This means that:
/// - initial memory allocated by a collection is bytewise initialized with the value `P`
/// - a region of memory occupied by an element is bytewise initialized with the value `P`
///   when the element is moved out, unless the region is immediately overwritten with another
///   element
/// - when a collection is dropped all memory of elements dropped with the collection is
///   bytewise initialized with the value `P`
pub struct Pattern<const P: u8>;

/// Zeroed spare memory policy.
///
/// This is a friendly alias for [`Pattern`] using zero as the pattern byte.
pub type Zeroed = Pattern<0>;

impl<T> SpareMemoryPolicy<T> for Uninitialized {}

impl<T> private::SpareMemoryPolicyBase<T> for Uninitialized {
    #[inline]
    unsafe fn init(_dst: *mut T, _count: usize) {
        // noop
    }
}

impl<T, const P: u8> SpareMemoryPolicy<T> for Pattern<P> {}

impl<T, const P: u8> private::SpareMemoryPolicyBase<T> for Pattern<P> {
    #[inline]
    unsafe fn init(dst: *mut T, count: usize) {
        ptr::write_bytes(dst, P, count)
    }
}
