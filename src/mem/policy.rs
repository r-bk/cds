use crate::sealed::Sealed;
use core::ptr;

/// Defines handling of spare memory in collections.
///
/// This includes both the initial allocation, and any memory that becomes free following removal of
/// an element from a collection (including removal forced by a [`drop`] of a collection).
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
/// `Uninitialized` is the fastest spare memory policy, because it is essentially a `noop`.
///
/// When used, spare memory is left intact:
/// - initial memory allocated by a collection remains uninitialized
/// - a region of memory occupied by an element of type `T` remains a byte-by-byte copy of the
/// element after its removal/move, until being overwritten with a value of another element
/// in the collection, or any other write made to that region after destruction of
/// the collection.
pub struct Uninitialized;

/// Pattern-initialized spare memory policy.
///
/// Written as `Pattern<P>`, pattern spare memory policy initializes every spare byte with the value
/// `P`.
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
