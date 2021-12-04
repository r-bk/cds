use core::ptr;

mod private {
    pub trait SpareMemoryPolicyBase<T> {
        unsafe fn init(dst: *mut T, count: usize);
        unsafe fn init_bytes(dst: *mut u8, count: usize);
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
/// applied to it. This may lead to having the old region of memory a byte-by-byte copy of
/// the memory the collection was moved to.
///
/// Currently the following policies are supported:
///
/// - [`Uninitialized`] does nothing with spare bytes
/// - [`Zeroed`] fills spare bytes with zeroes
/// - [`Pattern`] fills spare bytes with a specified value
///
/// [`drop`]: https://doc.rust-lang.org/core/mem/fn.drop.html
pub trait SpareMemoryPolicy<T>: private::SpareMemoryPolicyBase<T> {}

/// Uninitialized spare memory policy.
///
/// `Uninitialized` is the fastest spare memory policy, because it does absolutely nothing.
///
/// This means that:
/// - initial memory allocated by a collection remains uninitialized
/// - a region of memory occupied by an element remains a byte-by-byte copy of the
/// element after it is moved out, until the region is overwritten (if at all)
pub struct Uninitialized;

/// Pattern-initialized spare memory policy.
///
/// Written as `Pattern<P>`, pattern spare memory policy initializes every spare byte with the value
/// `P`.
///
/// This means that:
/// - initial memory allocated by a collection is byte-by-byte initialized with the value `P`
/// - a region of memory occupied by an element is byte-by-byte initialized with the value `P`
///   when the element is moved out, unless the region is immediately overwritten with another
///   element
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
    #[inline]
    unsafe fn init_bytes(_dst: *mut u8, _count: usize) {
        // noop
    }
}

impl<T, const P: u8> SpareMemoryPolicy<T> for Pattern<P> {}

impl<T, const P: u8> private::SpareMemoryPolicyBase<T> for Pattern<P> {
    #[inline]
    unsafe fn init(dst: *mut T, count: usize) {
        ptr::write_bytes(dst, P, count)
    }

    #[inline]
    unsafe fn init_bytes(dst: *mut u8, count: usize) {
        ptr::write_bytes(dst, P, count);
    }
}
