use crate::sealed::Sealed;
use core::ptr;

/// Defines handling of spare memory in collections.
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
pub trait SpareMemoryPolicy<T>: Sealed {
    /// Initialize spare memory of `count` elements of type `T` starting at `dst`.
    ///
    /// # Safety
    ///
    /// Behavior is undefined if any of the following conditions are violated:
    ///
    /// - `dst` must be [`valid`] for writes of `count * size_of::<T>()` bytes
    /// - `dst` must be properly aligned
    /// - `dst` must point to unoccupied memory
    ///
    /// Using a region of memory typed as `T` that contains an invalid value of `T` is undefined
    /// behavior. Thus, a collection must ensure that only free memory is handled with
    /// spare memory policy.
    ///
    /// [`valid`]: https://doc.rust-lang.org/core/ptr/index.html#safety
    unsafe fn init(dst: *mut T, count: usize);

    /// Initializes `count` bytes of spare memory starting at `dst`.
    ///
    /// # Safety
    ///
    /// Behavior is undefined if any of the following conditions are violated:
    ///
    /// - `dst` must be [`valid`] for writes of `count` bytes
    /// - `dst` must be properly aligned
    /// - `dst` must point to unoccupied memory
    ///
    /// Using a region of memory typed as `T` that contains an invalid value of `T` is undefined
    /// behavior. Thus, a collection must ensure that only free memory is handled with
    /// spare memory policy.
    ///
    /// [`valid`]: https://doc.rust-lang.org/core/ptr/index.html#safety
    unsafe fn init_bytes(dst: *mut u8, count: usize);
}

/// Uninitialized spare memory policy.
///
/// `Uninitialized` is the fastest spare memory policy, because it does absolutely nothing.
///
/// This means that:
/// - initial memory allocated by a collection remains uninitialized
/// - a region of memory occupied by an element of type `T` remains a byte-by-byte copy of the
/// element until being overwritten (if at all)
pub struct Uninitialized;

/// Pattern-initialized spare memory policy.
///
/// Written as `Pattern<P>`, pattern spare memory policy initializes every spare byte with the value
/// `P`.
///
/// This means that:
/// - initial memory allocated by a collection is byte-by-byte initialized with the value `P`
/// - a region of memory occupied by an element is byte-by-byte initialized with the value `P`
///   when the element is moved out of the region, unless being immediately overwritten with another
///   element
pub struct Pattern<const P: u8>;

/// Zeroed spare memory policy.
///
/// This is a friendly alias for [`Pattern`] using zero as the pattern byte.
pub type Zeroed = Pattern<0>;

impl Sealed for Uninitialized {}
impl<const P: u8> Sealed for Pattern<P> {}

impl<T> SpareMemoryPolicy<T> for Uninitialized {
    #[inline]
    unsafe fn init(_dst: *mut T, _count: usize) {
        // noop
    }
    #[inline]
    unsafe fn init_bytes(_dst: *mut u8, _count: usize) {
        // noop
    }
}

impl<T, const P: u8> SpareMemoryPolicy<T> for Pattern<P> {
    #[inline]
    unsafe fn init(dst: *mut T, count: usize) {
        ptr::write_bytes(dst, P, count)
    }

    #[inline]
    unsafe fn init_bytes(dst: *mut u8, count: usize) {
        ptr::write_bytes(dst, P, count);
    }
}
