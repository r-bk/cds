//! A vector with "small size" optimization.

use crate::{
    len::{LengthType, Usize},
    mem::{errors::ReservationError, SpareMemoryPolicy, Uninitialized},
};
use ::alloc::alloc::{self, Layout};
use core::{
    marker::PhantomData,
    mem,
    ops::{Bound, RangeBounds},
    ptr, slice,
};

mod buffer;
use buffer::SetLenOnDrop;

mod retain;
use retain::RetainGuard;

const DOHAE: bool = true; // call `handle_allocation_error`
const NOHAE: bool = false; // do not call `handle_allocation_error`

/// A continuous growable array with "small size" optimization.
///
/// Written as `SmallVec<T, C, L, SM>`, small vector stores elements of type `T`, has local
/// capacity to store up to `C` elements without allocating a heap buffer, uses `L` as
/// [`length type`], and `SM` as [`spare memory policy`].
///
/// `SmallVec` is mostly compatible with the standard [`Vec`] interface. Moreover, `SmallVec`
/// provides non-panic versions of various methods that in [`Vec`] may panic. In particular,
/// methods which, possibly implicitly, (re)allocate the heap buffer and may fail to reserve more
/// capacity have `try_` counterparts which return an error instead of panicking.
///
/// The "small size" optimization adds a fixed amount of local capacity to the `SmallVec` struct
/// itself. It allows storing up to `C` elements there, without allocating a heap buffer.
/// This may speed up a program, as usually heap allocation is a costly operation. Note, however,
/// that the size of a `SmallVec` struct is proportional to `C`, which may become inefficient for
/// high values of `C` in some use cases.
///
/// Similar to the standard vector, `SmallVec`'s total capacity in bytes is limited by
/// [`isize::MAX`]. Any attempt to allocate more memory fails with capacity overflow error, or a
/// panic in methods that allow it.
///
/// Unlike the standard vector, `SmallVec` doesn't allow customization of the memory allocator via a
/// generic parameter. It always uses the global memory allocator via [`alloc`] and [`realloc`]
/// methods. Support for custom allocators is planned to be added when the [`Allocator`] trait
/// becomes stable.
///
/// `SmallVec` calls [`handle_alloc_error`] when the memory allocator fails in methods allowed to
/// panic. The non-panic methods always return an appropriate `Result`, and [`handle_alloc_error`]
/// is not called.
///
/// [`spare memory policy`]: SpareMemoryPolicy
/// [`length type`]: LengthType
/// [`alloc`]: alloc::alloc
/// [`realloc`]: alloc::realloc
/// [`Allocator`]: alloc::Allocator
/// [`handle_alloc_error`]: alloc::handle_alloc_error
///
/// # Examples
///
/// ```rust
/// # use cds::{smallvec::SmallVec, len::U8};
/// let mut v = SmallVec::<u64, 4, U8>::new();
/// assert_eq!(v, []);
/// assert!(v.is_local());
///
/// v.push(1);
/// v.push(2);
///
/// assert_eq!(v, [1, 2]);
/// assert!(v.is_local());
///
/// v[0] = 7;
/// assert_eq!(v, [7, 2]);
///
/// v.extend(3..6);
/// assert_eq!(v, [7, 2, 3, 4, 5]);
/// assert!(v.is_heap());
///
/// for e in &v {
///     println!("{}", e);
/// }
/// ```
pub struct SmallVec<T, const C: usize, L = Usize, SM = Uninitialized>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    /// The buffer
    buf: buffer::Buffer<T, C, L, SM>,

    /// The length of small-vec when local; the capacity of the buffer when on heap
    capacity: L,

    phantom: PhantomData<T>,
}

impl<T, const C: usize, L, SM> SmallVec<T, C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    /// Checks if small-vector uses a heap buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::smallvec::SmallVec;
    /// type V = SmallVec<i32, 5>;
    /// let mut v = V::new();
    /// assert_eq!(v.is_heap(), false);  // <-- capacity <= C; local buffer is used
    /// v.reserve(10);                   // <-- capacity > C; a heap buffer is allocated
    /// assert_eq!(v.is_heap(), true);
    /// ```
    #[inline]
    pub fn is_heap(&self) -> bool {
        (mem::size_of::<T>() != 0) && (self.capacity.as_usize() > C)
    }

    /// Checks if small-vector uses a local buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::smallvec::SmallVec;
    /// type V = SmallVec<i32, 5>;
    /// let mut v = V::new();
    /// assert_eq!(v.is_local(), true);  // <-- capacity <= C; local buffer is used
    /// v.reserve(10);                   // <-- capacity > C; a heap buffer is allocated
    /// assert_eq!(v.is_local(), false);
    /// ```
    #[inline]
    pub fn is_local(&self) -> bool {
        (mem::size_of::<T>() == 0) || (self.capacity.as_usize() <= C)
    }

    /// Creates a new empty small-vector.
    ///
    /// Small-vector doesn't allocate until required capacity exceeds `C`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::smallvec::SmallVec;
    /// let sv = SmallVec::<usize, 5>::new();
    /// assert!(sv.is_empty());
    /// assert_eq!(sv.capacity(), 5);
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            buf: buffer::Buffer::new(),
            capacity: L::new(0),
            phantom: PhantomData,
        }
    }

    /// Constructs an empty small-vector with the specified capacity.
    ///
    /// Note that if `capacity < C` the capacity of the created small-vector is `C`.
    ///
    /// For [zero sized types] the capacity of the created small-vector equals the maximal value
    /// supported by the length-type `L`.
    ///
    /// # Panics
    ///
    /// See [`reserve_exact`] for panic conditions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::smallvec::SmallVec;
    /// type V = SmallVec<i32, 5>;         // <-- C == 5
    ///
    /// let v = V::with_capacity(3);       // <-- capacity <= C
    /// assert_eq!(v.capacity(), 5);       // <-- effective capacity is C
    /// assert_eq!(v.is_heap(), false);    // <-- the small-vector uses its local buffer
    /// assert!(v.is_empty());
    ///
    /// let v = V::with_capacity(10);      // <-- capacity > C
    /// assert_eq!(v.capacity(), 10);      // <-- effective capacity is the requested value
    /// assert_eq!(v.is_heap(), true);     // <-- the small-vector uses a heap buffer
    /// assert!(v.is_empty());
    /// ```
    ///
    /// [`reserve_exact`]: SmallVec::reserve_exact
    /// [zero sized types]: https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        let mut v = Self::new();
        v.reserve_exact(capacity);
        v
    }

    /// Returns the capacity of the small-vector.
    ///
    /// The capacity of a small-vector is the number of elements it can hold without reallocating
    /// the buffer.
    ///
    /// When a new small-vector is created, its capacity equals `C`. The capacity grows implicitly
    /// when more elements are added to the vector. The capacity can be pre-allocated via the
    /// [`reserve`] and [`reserve_exact`] methods.
    ///
    /// Note that, when `T` is a [zero sized type], the capacity is always `L::MAX` (even if
    /// `C` is smaller), and the buffer is never allocated on the heap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::small_vec;
    /// let mut v = small_vec![4; u64];
    /// assert_eq!(v.capacity(), 4);
    /// v.extend(0..6);
    /// assert_eq!(v, [0, 1, 2, 3, 4, 5]);
    /// assert_eq!(v.capacity(), 8);
    ///
    /// ```
    ///
    /// [zero sized type]: https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts\
    /// [`reserve`]: SmallVec::reserve
    /// [`reserve_exact`]: SmallVec::reserve_exact
    #[inline]
    pub fn capacity(&self) -> usize {
        if mem::size_of::<T>() == 0 {
            L::MAX
        } else {
            self.capacity.as_usize().max(C)
        }
    }

    /// Returns the spare capacity size of the small-vector.
    ///
    /// The spare capacity size of a small-vector is the number of elements it can hold, in addition
    /// to already held ones, without reallocating the buffer.
    ///
    /// This is equivalent to `capacity - len`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::small_vec;
    /// let mut v = small_vec![10; u64; 1, 2, 3];
    /// assert_eq!(v.capacity(), 10);
    /// assert_eq!(v.spare_capacity(), 7);
    /// assert_eq!(v.len(), 3);
    /// ```
    #[inline]
    pub fn spare_capacity(&self) -> usize {
        if mem::size_of::<T>() == 0 {
            L::MAX - self.capacity.as_usize()
        } else {
            let cap = self.capacity.as_usize();
            if cap <= C {
                C - cap
            } else {
                cap - self.buf.heap_len().as_usize()
            }
        }
    }

    /// Returns the remaining spare capacity of the small-vector as a slice of `MaybeUninit<T>`.
    ///
    /// The returned slice can be used to fill the small-vector with data (e.g. by reading from a
    /// file) before marking the data as initialized using the [`set_len`] method.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::small_vec;
    /// let mut v = small_vec![32; 1, 2];   // <-- a small-vector for IO of 32 elements
    /// assert_eq!(v, [1, 2]);
    ///
    /// let spare_capacity = v.spare_capacity_mut();
    /// spare_capacity[0].write(3);         // <-- read another 2 elements into the small-vector
    /// spare_capacity[1].write(4);
    ///
    /// unsafe { v.set_len(v.len() + 2) };  // <-- reflect the new size
    /// assert_eq!(v, [1, 2, 3, 4]);
    /// ```
    ///
    /// [`set_len`]: SmallVec::set_len
    #[inline]
    pub fn spare_capacity_mut(&mut self) -> &mut [mem::MaybeUninit<T>] {
        if mem::size_of::<T>() == 0 {
            unsafe {
                slice::from_raw_parts_mut(
                    self.buf.local_mut_ptr().cast(),
                    L::MAX - self.capacity.as_usize(),
                )
            }
        } else {
            let cap = self.capacity.as_usize();
            unsafe {
                let (spare_size, p) = if cap <= C {
                    (C - cap, self.buf.local_mut_ptr().add(cap))
                } else {
                    let (len, p) = self.buf.heap_len_mut_p();
                    let len = len.as_usize();
                    (cap - len, p.add(len))
                };
                slice::from_raw_parts_mut(p.cast(), spare_size)
            }
        }
    }

    /// Returns small-vector content as a slice of `T`, along with the remaining spare capacity of
    /// the small-vector as a slice of `MaybeUninit<T>`.
    ///
    /// The returned spare capacity slice can be used to fill the small-vector with data
    /// (e.g. by reading from a file) before marking the data as initialized using the [`set_len`]
    /// method.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::small_vec;
    /// let mut v = small_vec![32; 1, 2];   // <-- a small-vector for IO of 32 elements
    ///
    /// let (init, spare) = v.split_at_spare_mut();
    /// assert_eq!(init, &[1, 2]);
    ///
    /// assert_eq!(spare.len(), 30);        // <-- read another 2 elements into the small-vector
    /// spare[0].write(3);
    /// spare[1].write(4);
    ///
    /// unsafe { v.set_len(v.len() + 2) };  // <-- reflect the new size
    /// assert_eq!(v, [1, 2, 3, 4]);
    /// ```
    ///
    /// [`set_len`]: SmallVec::set_len
    #[inline]
    pub fn split_at_spare_mut(&mut self) -> (&mut [T], &mut [mem::MaybeUninit<T>]) {
        let cap = self.capacity.as_usize();
        if mem::size_of::<T>() == 0 {
            let p = self.buf.local_mut_ptr();
            unsafe {
                (
                    slice::from_raw_parts_mut(p, cap),
                    slice::from_raw_parts_mut(p.add(cap).cast(), L::MAX - cap),
                )
            }
        } else if cap <= C {
            let p = self.buf.local_mut_ptr();
            unsafe {
                (
                    slice::from_raw_parts_mut(p, cap),
                    slice::from_raw_parts_mut(p.add(cap).cast(), C - cap),
                )
            }
        } else {
            let (len, p) = self.buf.heap_len_mut_p();
            let len = len.as_usize();
            unsafe {
                (
                    slice::from_raw_parts_mut(p, len),
                    slice::from_raw_parts_mut(p.add(len).cast(), cap - len),
                )
            }
        }
    }

    /// Checks if the small-vector has spare capacity.
    ///
    /// This is equivalent to `spare_capacity > 0`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::small_vec;
    /// let mut v = small_vec![4; 1, 2];
    /// assert_eq!(v.capacity(), 4);
    /// assert_eq!(v.has_spare_capacity(), true);
    /// v.extend(3..5);
    /// assert_eq!(v.has_spare_capacity(), false);
    /// ```
    #[inline]
    pub fn has_spare_capacity(&self) -> bool {
        if mem::size_of::<T>() == 0 {
            L::MAX > self.capacity.as_usize()
        } else {
            let cap = self.capacity.as_usize();
            if cap <= C {
                cap < C
            } else {
                self.buf.heap_len().as_usize() < cap
            }
        }
    }

    /// Returns the length of small-vector.
    ///
    /// The length of small-vector is the number of elements it currently holds.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::small_vec;
    /// let mut v = small_vec![8; 1, 2];
    /// assert_eq!(v.len(), 2);
    /// v.push(20);
    /// assert_eq!(v.len(), 3);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        let cap = self.capacity.as_usize();
        if mem::size_of::<T>() == 0 || cap <= C {
            cap
        } else {
            self.buf.heap_len().as_usize()
        }
    }

    /// Checks if the small-vector is empty.
    ///
    /// This is equivalent to `len == 0`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::small_vec;
    /// let mut v = small_vec![8; u64];
    /// assert_eq!(v.is_empty(), true);
    /// v.push(2);
    /// assert_eq!(v.is_empty(), false);
    /// v.clear();
    /// assert_eq!(v.is_empty(), true);
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Checks if the small-vector is full.
    ///
    /// This is equivalent to `len == capacity`.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::small_vec;
    /// let mut v = small_vec![3; u64; 1, 2];
    /// assert_eq!(v.capacity(), 3);
    /// assert_eq!(v.is_full(), false);
    /// v.push(3);
    /// assert_eq!(v.is_full(), true);
    /// ```
    #[inline]
    pub fn is_full(&self) -> bool {
        let cap = self.capacity.as_usize();
        if mem::size_of::<T>() == 0 {
            cap == L::MAX
        } else if cap <= C {
            cap == C
        } else {
            cap == self.buf.heap_len().as_usize()
        }
    }

    /// Forces the length of the small-vector to `len`.
    ///
    /// Note that this method doesn't [`drop`] elements, which may lead to a resource leak if
    /// `len < len()` and `T` has a custom [`Drop`] implementation. See [`truncate`] for a
    /// method that handles small-vector truncation properly.
    ///
    /// # Safety
    ///
    /// - `len` must be less than or equal to current small-vector's [`capacity`]
    /// - the elements at `old_len..len` must be initialized
    ///
    /// # Panics
    ///
    /// This method uses debug assertions to verify that `len` is in bounds.
    ///
    /// [`drop`]: core::mem::drop
    /// [`Drop`]: core::ops::Drop
    /// [`truncate`]: SmallVec::truncate
    /// [`capacity`]: SmallVec::capacity
    #[inline]
    pub unsafe fn set_len(&mut self, len: usize) {
        if mem::size_of::<T>() == 0 {
            debug_assert!(len <= L::MAX);
            self.capacity.set(len);
        } else {
            let cap = self.capacity.as_usize();
            if cap <= C {
                debug_assert!(len <= C);
                self.capacity.set(len);
            } else {
                debug_assert!(len <= cap);
                self.buf.set_heap_len(L::new(len))
            }
        }
    }

    /// Extracts a slice containing the entire small-vector.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        let (len, p) = self.len_p();
        unsafe { slice::from_raw_parts(p, len.as_usize()) }
    }

    /// Extracts a mutable slice of the entire small-vector.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        let (len, p) = self.len_mut_p();
        unsafe { slice::from_raw_parts_mut(p, len.as_usize()) }
    }

    /// Returns a raw pointer to the small-vector's buffer.
    ///
    /// The returned value may point either to the local buffer or to the heap.
    #[inline]
    pub fn as_ptr(&self) -> *const T {
        if self.is_local() {
            self.buf.local_ptr()
        } else {
            self.buf.heap_ptr()
        }
    }

    /// Returns an unsafe mutable pointer to the small-vector's buffer.
    ///
    /// The returned value may point either to the local buffer or to the heap.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        if self.is_local() {
            self.buf.local_mut_ptr()
        } else {
            self.buf.heap_mut_ptr()
        }
    }

    /// Returns an iterator over the slice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::small_vec;
    /// let v = small_vec![3; 1, 2];
    /// let mut iterator = v.iter();
    /// assert_eq!(iterator.next(), Some(&1));
    /// assert_eq!(iterator.next(), Some(&2));
    /// assert_eq!(iterator.next(), None);
    /// ```
    #[inline]
    pub fn iter(&self) -> slice::Iter<'_, T> {
        self.as_slice().iter()
    }

    /// Returns an iterator over the slice that allows modifying each value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::small_vec;
    /// let mut v = small_vec![3; 1, 2];
    /// for e in v.iter_mut() {
    ///     *e *= 2;
    /// }
    /// assert_eq!(v, [2, 4]);
    /// ```
    #[inline]
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, T> {
        self.as_mut_slice().iter_mut()
    }

    /// Returns (len, ptr)
    #[inline]
    fn len_p(&self) -> (L, *const T) {
        let cap = self.capacity;
        if mem::size_of::<T>() == 0 || cap.as_usize() <= C {
            (cap, self.buf.local_ptr())
        } else {
            self.buf.heap_len_p()
        }
    }

    /// Returns (len, mut_ptr)
    #[inline]
    fn len_mut_p(&mut self) -> (L, *mut T) {
        let cap = self.capacity;
        if mem::size_of::<T>() == 0 || cap.as_usize() <= C {
            (cap, self.buf.local_mut_ptr())
        } else {
            self.buf.heap_len_mut_p()
        }
    }

    /// Reserves capacity for at least the given number of additional elements.
    ///
    /// # Panics
    ///
    /// This method panics on any of the following conditions:
    /// - the total capacity overflows the length type `L::MAX`
    /// - the total capacity in bytes overflows `isize::MAX`
    /// - memory allocation fails ([`alloc::handle_alloc_error`] is called)
    ///
    /// See [`try_reserve`] for a method that returns [`ReservationError`] instead.
    ///
    /// [`try_reserve`]: SmallVec::try_reserve
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.try_reserve_impl::<DOHAE>(additional)
            .expect("smallvec reserve failed");
    }

    /// Tries to reserve capacity for at least the given number of additional elements.
    ///
    /// This is a non-panic version of [`reserve`].
    ///
    /// [`reserve`]: SmallVec::reserve
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), ReservationError> {
        self.try_reserve_impl::<NOHAE>(additional).map(|_| ())
    }

    #[inline(never)]
    fn try_reserve_impl<const HAE: bool>(
        &mut self,
        additional: usize,
    ) -> Result<(&mut L, *mut T), ReservationError> {
        self.reserve_core::<_, HAE>(additional, |l, a| {
            Ok(l.checked_add_usize(a)
                .ok_or(ReservationError::CapacityOverflow)?
                .next_power_of_two_or_max())
        })
    }

    /// Reserves the minimum amount of capacity space for a given number of additional elements.
    ///
    /// # Panics
    ///
    /// This method panics on any of the following conditions:
    /// - the total capacity overflows the length type `L::MAX`
    /// - the total capacity in bytes overflows `isize::MAX`
    /// - memory allocation fails ([`alloc::handle_alloc_error`] is called)
    ///
    /// See [`try_reserve_exact`] for a method that returns [`ReservationError`] instead.
    ///
    /// [`try_reserve_exact`]: SmallVec::try_reserve_exact
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.try_reserve_exact_impl::<DOHAE>(additional)
            .expect("smallvec reserve_exact failed");
    }

    /// Tries to reserve minimum amount of space for a given number of additional elements.
    ///
    /// This method tries to reserve the minimum amount of memory required for `additional`
    /// elements. Upon successful completion the capacity of small-vector is greater or equal
    /// `len + additional`. The method does nothing if the capacity is already sufficient.
    ///
    /// This is a non-panic version of [`reserve_exact`].
    ///
    /// [`reserve_exact`]: SmallVec::reserve_exact
    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), ReservationError> {
        self.try_reserve_exact_impl::<NOHAE>(additional).map(|_| ())
    }

    #[inline(never)]
    fn try_reserve_exact_impl<const HAE: bool>(
        &mut self,
        additional: usize,
    ) -> Result<(&mut L, *mut T), ReservationError> {
        self.reserve_core::<_, HAE>(additional, |l, a| {
            l.checked_add_usize(a)
                .ok_or(ReservationError::CapacityOverflow)
        })
    }

    #[inline]
    fn reserve_core<F, const HAE: bool>(
        &mut self,
        additional: usize,
        nc: F,
    ) -> Result<(&mut L, *mut T), ReservationError>
    where
        F: FnOnce(L, usize) -> Result<L, ReservationError>,
    {
        let cap = self.capacity.as_usize();
        if mem::size_of::<T>() == 0 {
            let len = cap;
            let cap = L::MAX;
            if cap - len >= additional {
                Ok((&mut self.capacity, self.buf.local_mut_ptr()))
            } else {
                Err(ReservationError::CapacityOverflow)
            }
        } else if cap <= C {
            let len = cap;
            let cap = C;
            if cap - len >= additional {
                return Ok((&mut self.capacity, self.buf.local_mut_ptr()));
            }

            let new_cap = nc(self.capacity, additional)?;
            debug_assert!(new_cap > C);
            debug_assert!(new_cap > cap);

            let p: *mut T = unsafe {
                let new_layout = Layout::array::<T>(new_cap.as_usize())
                    .map_err(|_| ReservationError::CapacityOverflow)?;
                if new_layout.size() > isize::MAX as usize {
                    return Err(ReservationError::CapacityOverflow);
                }

                let tmp = alloc::alloc(new_layout);
                if tmp.is_null() {
                    if HAE {
                        alloc::handle_alloc_error(new_layout);
                    }
                    return Err(ReservationError::AllocError { layout: new_layout });
                }

                // if spare memory policy is a noop do not copy the old spare memory
                let n = if SM::NOOP { len } else { cap };
                ptr::copy_nonoverlapping(self.buf.local_ptr(), tmp.cast(), n);
                tmp.cast()
            };

            if !SM::NOOP {
                unsafe {
                    // initialize the new spare memory only; old spare memory was preserved
                    SM::init(p.add(cap), new_cap.as_usize() - cap);
                    SM::init(self.buf.local_mut_ptr(), len)
                }
            }

            self.buf.set_heap(p, self.capacity);
            self.capacity = new_cap;
            Ok(self.buf.heap_mut_len_mut_p())
        } else {
            let len = self.buf.heap_len();
            if cap - len.as_usize() >= additional {
                return Ok(self.buf.heap_mut_len_mut_p());
            }

            let new_cap = nc(len, additional)?;
            debug_assert!(new_cap > cap);
            let old_p = self.buf.heap_mut_ptr();
            let old_array_size = mem::size_of::<T>() * cap;
            let old_layout =
                unsafe { Layout::from_size_align_unchecked(old_array_size, mem::align_of::<T>()) };
            let new_layout = Layout::array::<T>(new_cap.as_usize())
                .map_err(|_| ReservationError::CapacityOverflow)?;
            if new_layout.size() > isize::MAX as usize {
                return Err(ReservationError::CapacityOverflow);
            }

            let p: *mut T = if SM::NOOP {
                unsafe {
                    let tmp = alloc::realloc(old_p.cast(), old_layout, new_layout.size());
                    if tmp.is_null() {
                        if HAE {
                            alloc::handle_alloc_error(new_layout);
                        }
                        return Err(ReservationError::AllocError { layout: new_layout });
                    }
                    tmp.cast()
                }
            } else {
                unsafe {
                    let tmp = alloc::alloc(new_layout);
                    if tmp.is_null() {
                        if HAE {
                            alloc::handle_alloc_error(new_layout);
                        }
                        return Err(ReservationError::AllocError { layout: new_layout });
                    }

                    // copy the old buffer including its spare memory
                    ptr::copy_nonoverlapping(old_p.cast(), tmp, old_layout.size());
                    SM::init((tmp as *mut T).add(cap), new_cap.as_usize() - cap);
                    SM::init(old_p, len.as_usize());
                    alloc::dealloc(old_p.cast(), old_layout);
                    tmp.cast()
                }
            };

            self.buf.set_heap_ptr(p);
            self.capacity = new_cap;
            Ok(self.buf.heap_mut_len_mut_p())
        }
    }

    /// Appends an element to the back of the small-vector.
    ///
    /// # Panics
    ///
    /// See [`reserve`] for panic conditions.
    ///
    /// See [`try_push`] for a non-panic version of this method.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::small_vec;
    /// let mut v = small_vec![2; u64];
    /// assert_eq!(v, []);
    /// v.push(1);
    /// v.push(2);
    /// assert_eq!(v, [1, 2]);
    /// ```
    ///
    /// [`reserve`]: SmallVec::reserve
    /// [`try_push`]: SmallVec::try_push
    #[inline]
    pub fn push(&mut self, e: T) {
        self.try_push_impl::<DOHAE>(e)
            .expect("smallvec push failed")
    }

    /// Tries to append an element to the back of the small-vector.
    ///
    /// This is a non-panic version of [`push`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::{small_vec, mem::errors::ReservationError};
    /// # fn foo() -> Result<(), ReservationError> {
    /// let mut v = small_vec![2; u64];
    /// assert_eq!(v, []);
    /// v.try_push(10)?;
    /// v.try_push(20)?;
    /// assert_eq!(v, [10, 20]);
    /// # Ok(())
    /// # }
    /// # foo();
    /// ```
    ///
    /// [`push`]: SmallVec::push
    #[inline]
    pub fn try_push(&mut self, e: T) -> Result<(), ReservationError> {
        self.try_push_impl::<NOHAE>(e)
    }

    #[inline]
    #[allow(clippy::comparison_chain)]
    fn try_push_impl<const HAE: bool>(&mut self, e: T) -> Result<(), ReservationError> {
        let cap = self.capacity.as_usize();
        if mem::size_of::<T>() == 0 {
            if cap < L::MAX {
                self.capacity.add_assign(1);
                unsafe { self.buf.local_mut_ptr().write(e) };
            } else {
                return Err(ReservationError::CapacityOverflow);
            }
            Ok(())
        } else {
            let len;
            let p;
            if cap < C {
                p = self.buf.local_mut_ptr();
                len = cap;
                self.capacity.add_assign(1);
            } else if cap > C {
                len = self.buf.heap_len().as_usize();
                if len == cap {
                    self.try_reserve_impl::<HAE>(1)?;
                }
                p = self.buf.heap_mut_ptr();
                self.buf.heap_len_add_assign(1);
            } else {
                self.try_reserve_impl::<HAE>(1)?;
                p = self.buf.heap_mut_ptr();
                len = self.buf.heap_len().as_usize();
                self.buf.heap_len_add_assign(1);
            }
            unsafe {
                p.add(len).write(e);
            }
            Ok(())
        }
    }

    /// Removes the last element from a small-vector and returns it, or [`None`] if it is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::small_vec;
    /// let mut v = small_vec![3; 10, 13];
    /// assert_eq!(v.pop(), Some(13));
    /// assert_eq!(v.pop(), Some(10));
    /// assert_eq!(v.pop(), None);
    /// assert_eq!(v, []);
    /// ```
    #[inline]
    #[allow(clippy::comparison_chain)]
    pub fn pop(&mut self) -> Option<T> {
        if mem::size_of::<T>() == 0 {
            let len = self.capacity.as_usize();
            if len > 0 {
                self.capacity.sub_assign(1);
                unsafe { Some(self.buf.local_ptr().read()) }
            } else {
                None
            }
        } else {
            let cap = self.capacity.as_usize();
            if cap <= C {
                if cap > 0 {
                    let new_len = cap - 1;
                    self.capacity = L::new(new_len);
                    unsafe {
                        let p = self.buf.local_mut_ptr().add(new_len);
                        let e = p.read();
                        SM::init(p, 1);
                        Some(e)
                    }
                } else {
                    None
                }
            } else {
                let len = self.buf.heap_len().as_usize();
                if len > 0 {
                    let new_len = len - 1;
                    self.buf.set_heap_len(L::new(new_len));
                    unsafe {
                        let p = self.buf.heap_mut_ptr().add(new_len);
                        let e = p.read();
                        SM::init(p, 1);
                        Some(e)
                    }
                } else {
                    None
                }
            }
        }
    }

    /// Shortens the small-vector, keeping the first `len` elements and dropping the rest.
    ///
    /// If `len` is greater than small-vector's current length, this has no effect.
    ///
    /// # Safety
    ///
    /// Spare memory policy is invoked only if all truncated elements drop successfully. I.e, if
    /// any of the truncated elements panics during drop, spare memory policy isn't invoked
    /// at all, including on successfully dropped elements.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::small_vec;
    /// let mut v = small_vec![8; 1, 2, 3];
    /// assert_eq!(v, [1, 2, 3]);
    /// v.truncate(1);
    /// assert_eq!(v, [1]);
    /// v.truncate(2);
    /// assert_eq!(v, [1]);
    /// ```
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        let my_len = self.len();

        if len < my_len {
            unsafe {
                // `drop` of any of the truncated slots may panic, which may trigger destruction
                // of `self`. Thus, update `self.len` *before* calling `drop_in_place` to avoid
                // a possible double-drop of a truncated slot.
                self.set_len(len);

                // create a slice of truncated slots
                let s = slice::from_raw_parts_mut(self.as_mut_ptr().add(len), my_len - len);

                // `drop_in_place` drops every slot in the slice. If one slot panics, it will first
                // try to drop the rest and only then re-raise the panic.
                // If more than one slot panic, the program aborts.
                ptr::drop_in_place(s);

                // TODO: invoke SM-policy on every successfully dropped element, even if one panics

                // invoke spare memory policy
                SM::init(s.as_mut_ptr(), s.len());
            }
        }
    }

    /// Clears the small-vector, dropping all values.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::small_vec;
    /// let mut v = small_vec![16; 1, 2, 3];
    /// assert_eq!(v, [1, 2, 3]);
    /// v.clear();
    /// assert_eq!(v, []);
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.truncate(0)
    }

    /// Creates a small-vector from an iterator.
    ///
    /// Returns [`ReservationError`] if memory allocation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::{
    /// #     smallvec::SmallVec,
    /// #     len::U8, mem::{Uninitialized, errors::ReservationError},
    /// # };
    /// # fn example() -> Result<(), ReservationError> {
    /// type SV = SmallVec<usize, 3, U8, Uninitialized>;
    /// let a = [1, 2, 3];
    /// let v = SV::try_from_iter(a.iter().filter(|x| **x % 2 == 0).cloned())?;
    /// assert_eq!(v, [2]);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn try_from_iter<I>(iter: I) -> Result<Self, ReservationError>
    where
        I: IntoIterator<Item = T>,
    {
        Self::try_from_iter_impl::<I, NOHAE>(iter)
    }

    #[inline]
    fn try_from_iter_impl<I, const HAE: bool>(iter: I) -> Result<Self, ReservationError>
    where
        I: IntoIterator<Item = T>,
    {
        let mut tmp = Self::new();
        tmp.try_extend_impl::<I, HAE>(iter)?;
        Ok(tmp)
    }

    #[inline]
    fn try_extend_impl<I, const HAE: bool>(&mut self, iter: I) -> Result<(), ReservationError>
    where
        I: IntoIterator<Item = T>,
    {
        let it = iter.into_iter();
        let (min, max) = it.size_hint();
        let cap = max.unwrap_or(min);

        let len = self.len();
        let mut g = SetLenOnDrop::new(self, len);
        g.sv.try_reserve_impl::<HAE>(cap)?;

        let mut cap = g.sv.capacity();
        let mut p = unsafe { g.sv.as_mut_ptr().add(len) };

        for e in it {
            unsafe {
                if g.len >= cap {
                    g.sv.set_len(g.len);
                    let (_, tmp_p) = g.sv.try_reserve_impl::<HAE>(1)?;
                    p = tmp_p.add(g.len);
                    cap = g.sv.capacity();
                }
                p.write(e);
                p = p.add(1);
                g.len += 1;
            }
        }

        drop(g);
        Ok(())
    }

    /// Inserts an element at position `index` within the small-vector, shifting all elements after it to the right.
    ///
    /// # Panics
    ///
    /// This method panics if `index > len`, or when capacity reservation error occurs.
    /// See [`reserve`] for more information.
    ///
    /// See [`try_insert`] for a non-panic version of this method.
    ///
    /// [`reserve`]: SmallVec::reserve
    /// [`try_insert`]: SmallVec::try_insert
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::small_vec;
    /// let mut v = small_vec![4; usize; 1, 2, 4];
    /// v.insert(2, 3);
    /// assert_eq!(v, [1, 2, 3, 4]);
    /// v.insert(4, 5);
    /// assert_eq!(v, [1, 2, 3, 4, 5]);
    /// ```
    #[inline]
    pub fn insert(&mut self, index: usize, value: T) {
        self.try_insert_impl::<DOHAE>(index, value)
            .expect("smallvec insert failed")
    }

    /// Tries to insert an element at position `index` within the small-vector, shifting all elements after it to the right.
    ///
    /// Returns `InsertError::InvalidIndex` if `index > len`. Or `InsertError::ReservationError`
    /// if capacity reservation fails. See [`try_reserve`] for more information.
    ///
    /// This is a non-panic version of [`insert`].
    ///
    /// [`try_reserve`]: SmallVec::try_reserve
    /// [`insert`]: SmallVec::insert
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::{small_vec, smallvec::errors::InsertError};
    /// # fn foo() -> Result<(), InsertError> {
    /// let mut v = small_vec![4; usize; 1, 2, 4];
    /// v.try_insert(2, 3)?;
    /// assert_eq!(v, [1, 2, 3, 4]);
    /// v.try_insert(4, 5)?;
    /// assert_eq!(v, [1, 2, 3, 4, 5]);
    /// # Ok(())
    /// # }
    /// # foo().unwrap();
    /// ```
    #[inline]
    pub fn try_insert(&mut self, index: usize, value: T) -> Result<(), InsertError> {
        self.try_insert_impl::<NOHAE>(index, value)
    }

    #[inline]
    #[allow(clippy::comparison_chain)]
    fn try_insert_impl<const HAE: bool>(
        &mut self,
        index: usize,
        value: T,
    ) -> Result<(), InsertError> {
        let cap = self.capacity.as_usize();
        if mem::size_of::<T>() == 0 {
            if index > cap {
                Err(InsertError::InvalidIndex)
            } else if cap < L::MAX {
                self.capacity.add_assign(1);
                unsafe { self.buf.local_mut_ptr().write(value) };
                Ok(())
            } else {
                Err(InsertError::ReservationError(
                    ReservationError::CapacityOverflow,
                ))
            }
        } else {
            let len;
            let p;
            if cap < C {
                p = self.buf.local_mut_ptr();
                len = cap;
                if index > len {
                    return Err(InsertError::InvalidIndex);
                }
                self.capacity.add_assign(1);
            } else if cap > C {
                len = self.buf.heap_len().as_usize();
                if index > len {
                    return Err(InsertError::InvalidIndex);
                }
                if len == cap {
                    self.try_reserve_impl::<HAE>(1)
                        .map_err(InsertError::ReservationError)?;
                }
                p = self.buf.heap_mut_ptr();
                self.buf.heap_len_add_assign(1);
            } else {
                if index > cap {
                    return Err(InsertError::InvalidIndex);
                }
                self.try_reserve_impl::<HAE>(1)
                    .map_err(InsertError::ReservationError)?;
                p = self.buf.heap_mut_ptr();
                len = self.buf.heap_len().as_usize();
                self.buf.heap_len_add_assign(1);
            }
            unsafe {
                let p = p.add(index);
                ptr::copy(p, p.add(1), len - index);
                p.write(value);
            }
            Ok(())
        }
    }

    /// Removes and returns the element at position `index` within the small-vector, shifting
    /// all elements after it to the left.
    ///
    /// # Panics
    ///
    /// This method panics if `index` is out of bounds.
    ///
    /// See [`try_remove`] for a non-panic version of this method.
    ///
    /// [`try_remove`]: SmallVec::try_remove
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::small_vec;
    /// let mut v = small_vec![4; u64; 1, 2, 3];
    /// assert_eq!(v.remove(1), 2);
    /// assert_eq!(v, [1, 3]);
    /// ```
    ///
    /// The following example panics because `index` is out of bounds:
    ///
    /// ```should_panic
    /// # use cds::small_vec;
    /// let mut v = small_vec![4; u64; 1, 2, 3];
    /// v.remove(4);
    /// ```
    #[inline]
    pub fn remove(&mut self, index: usize) -> T {
        self.try_remove(index)
            .expect("smallvec remove: index out of bounds")
    }

    /// Tries to remove and return the element at position `index` within the small-vector, shifting
    /// all elements after it to the left.
    ///
    /// Returns `None` if `index` is out of bounds.
    ///
    /// This is a non-panic version of [`remove`].
    ///
    /// [`remove`]: SmallVec::remove
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::small_vec;
    /// let mut v = small_vec![4; u64; 1, 2, 3];
    /// assert_eq!(v.try_remove(1), Some(2));
    /// assert_eq!(v.try_remove(2), None);
    /// assert_eq!(v, [1, 3]);
    /// ```
    #[inline]
    pub fn try_remove(&mut self, index: usize) -> Option<T> {
        if mem::size_of::<T>() == 0 {
            let len = self.capacity.as_usize();
            if index < len {
                self.capacity.sub_assign(1);
                unsafe { Some(self.buf.local_ptr().read()) }
            } else {
                None
            }
        } else {
            let len;
            let p;
            let cap = self.capacity.as_usize();
            if cap <= C {
                if index < cap {
                    self.capacity.sub_assign(1);
                    p = self.buf.local_mut_ptr();
                    len = cap;
                } else {
                    return None;
                }
            } else {
                len = self.buf.heap_len().as_usize();
                if index < len {
                    self.buf.set_heap_len(L::new(len - 1));
                    p = self.buf.heap_mut_ptr();
                } else {
                    return None;
                }
            }
            unsafe {
                let p = p.add(index);
                let value = p.read();
                let to_copy = len - index - 1;
                ptr::copy(p.add(1), p, to_copy);
                SM::init(p.add(to_copy), 1);
                Some(value)
            }
        }
    }

    /// Removes an element at position `index` and returns it.
    ///
    /// The removed element is replaced by the last element of the small-vector.
    /// This does not preserve ordering, but is O(1).
    ///
    /// # Panics
    ///
    /// This method panics of `index` is out of bounds. See [`try_swap_remove`] for a non-panic
    /// version of this method.
    ///
    /// [`try_swap_remove`]: SmallVec::try_swap_remove
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::small_vec;
    /// let mut v = small_vec![4; u64; 1, 2, 3, 4];
    /// assert_eq!(v.swap_remove(1), 2);
    /// assert_eq!(v, [1, 4, 3]);
    /// ```
    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> T {
        self.try_swap_remove(index)
            .expect("smallvec swap_remove: index out of bounds")
    }

    /// Tries to remove an element at position `index` and returns it.
    ///
    /// Returns `None` if `index` is out of bounds.
    ///
    /// The removed element is replaced by the last element of the small-vector.
    /// This does not preserve ordering, but is O(1).
    ///
    /// This is a non-panic version of [`swap_remove`].
    ///
    /// [`swap_remove`]: SmallVec::swap_remove
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::small_vec;
    /// let mut v = small_vec![4; u64; 1, 2, 3, 4];
    /// assert_eq!(v.try_swap_remove(1), Some(2));
    /// assert_eq!(v.try_swap_remove(10), None);
    /// assert_eq!(v, [1, 4, 3]);
    /// ```
    #[inline]
    pub fn try_swap_remove(&mut self, index: usize) -> Option<T> {
        if mem::size_of::<T>() == 0 {
            let len = self.capacity.as_usize();
            if index < len {
                self.capacity.sub_assign(1);
                unsafe { Some(self.buf.local_ptr().read()) }
            } else {
                None
            }
        } else {
            let len;
            let p;
            let cap = self.capacity.as_usize();
            if cap <= C {
                if index < cap {
                    self.capacity.sub_assign(1);
                    p = self.buf.local_mut_ptr();
                    len = cap;
                } else {
                    return None;
                }
            } else {
                len = self.buf.heap_len().as_usize();
                if index < len {
                    self.buf.set_heap_len(L::new(len - 1));
                    p = self.buf.heap_mut_ptr();
                } else {
                    return None;
                }
            }
            unsafe {
                let p_last = p.add(len - 1);
                let p = p.add(index);
                let value = p.read();
                ptr::copy(p_last, p, 1);
                SM::init(p_last, 1);
                Some(value)
            }
        }
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` such that `f(&e)` returns `false`.
    /// This method operates in place, visiting each element exactly once in the original order,
    /// and preserves the order of the retained elements.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::small_vec;
    /// let mut v = small_vec![5; 0, 1, 2, 3, 4];
    /// assert_eq!(v, [0, 1, 2, 3, 4]);
    /// v.retain(|e| (*e & 1) != 0);
    /// assert_eq!(v, [1, 3]);
    /// ```
    #[inline]
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.retain_mut(|e| f(e))
    }

    /// Retains only the elements specified by the predicate, passing a mutable reference to it.
    ///
    /// In other words, remove all elements `e` such that `f(&mut e)` returns `false`.
    /// This method operates in place, visiting each element exactly once in the original order,
    /// and preserves the order of the retained elements.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::small_vec;
    /// let mut v = small_vec![5; 0, 1, 2, 3, 4];
    /// assert_eq!(v, [0, 1, 2, 3, 4]);
    /// v.retain_mut(|e| if (*e & 1) == 0 {
    ///    *e *= *e;
    ///    true
    /// } else {
    ///    false
    /// });
    /// assert_eq!(v, [0, 4, 16]);
    /// ```
    #[inline]
    pub fn retain_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        let mut g = RetainGuard {
            sv: self,
            len: 0,
            deleted: 0,
            processed: 0,
        };

        let cap = g.sv.capacity.as_usize();

        // set `len` to zero, to avoid double-drop of deleted items.
        // `len` is restored by RetainGuard.
        let (len, p) = if mem::size_of::<T>() == 0 || cap <= C {
            g.sv.capacity.set(0);
            (cap, g.sv.buf.local_mut_ptr())
        } else {
            let (l, p) = g.sv.buf.heap_len_mut_p();
            g.sv.buf.set_heap_len(L::new(0));
            (l.as_usize(), p)
        };

        g.len = len;

        unsafe {
            // no empty slot found yet, so there is nothing to move
            while g.processed < len {
                let item_mut_ref = &mut *p.add(g.processed);
                if !f(item_mut_ref) {
                    // update counters before drop_in_place, as it may panic
                    g.processed += 1;
                    g.deleted += 1;
                    ptr::drop_in_place(item_mut_ref);
                    break;
                }
                g.processed += 1;
            }

            // If there are items left to process, there must be an empty slot.
            // Move every retained slot to an empty one.
            while g.processed < len {
                let item_mut_ref = &mut *p.add(g.processed);
                if !f(item_mut_ref) {
                    // update counters before drop_in_place, as it may panic
                    g.processed += 1;
                    g.deleted += 1;
                    ptr::drop_in_place(item_mut_ref);
                } else {
                    ptr::copy_nonoverlapping(
                        item_mut_ref as *const _,
                        p.add(g.processed - g.deleted),
                        1,
                    );
                    g.processed += 1;
                }
            }
        }
    }

    /// Resizes the small-vector in-place so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, the small-vector is extended by the difference,
    /// with each additional slot filled with the result of calling the closure `f`.
    /// The return values from `f` will end up in the small-vector in the order they have been
    /// generated.
    ///
    /// If `new_len` is less than `len`, the small-vector is simply truncated.
    ///
    /// # Panics
    ///
    /// This method panics on capacity reservation errors. See [`reserve`] for more information.
    ///
    /// See [`try_resize_with`] for a non-panic version of this method.
    ///
    /// [`reserve`]: SmallVec::reserve
    /// [`try_resize_with`]: SmallVec::try_resize_with
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::small_vec;
    /// let mut v = small_vec![8; u64; 0, 0];
    /// assert_eq!(v, [0, 0]);
    /// let mut i = 9;
    /// v.resize_with(5, || { i += 1; i });
    /// assert_eq!(v, [0, 0, 10, 11, 12]);
    /// ```
    #[inline]
    pub fn resize_with<F>(&mut self, new_len: usize, f: F)
    where
        F: FnMut() -> T,
    {
        self.try_resize_with_impl::<F, DOHAE>(new_len, f)
            .expect("smallvec resize_with failed")
    }

    /// Tries to resize the small-vector in-place so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, the small-vector is extended by the difference,
    /// with each additional slot filled with the result of calling the closure `f`.
    /// The return values from `f` will end up in the small-vector in the order they have been
    /// generated.
    ///
    /// If `new_len` is less than `len`, the small-vector is simply truncated.
    ///
    /// This is a non-panic version of [`resize_with`].
    ///
    /// [`resize_with`]: SmallVec::resize_with
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::{small_vec, mem::errors::ReservationError};
    /// # fn foo() -> Result<(), ReservationError> {
    /// let mut v = small_vec![8; u64];
    /// assert_eq!(v, []);
    /// v.try_resize_with(5, || 5)?;
    /// assert_eq!(v, [5, 5, 5, 5, 5]);
    /// v.try_resize_with(5, || 0)?;
    /// assert_eq!(v, [5, 5, 5, 5, 5]);
    /// v.try_resize_with(2, || 0)?;
    /// assert_eq!(v, [5, 5]);
    /// # Ok(())
    /// # }
    /// # foo().unwrap();
    /// ```
    #[inline]
    pub fn try_resize_with<F>(&mut self, new_len: usize, f: F) -> Result<(), ReservationError>
    where
        F: FnMut() -> T,
    {
        self.try_resize_with_impl::<F, NOHAE>(new_len, f)
    }

    #[inline]
    #[allow(clippy::comparison_chain)]
    fn try_resize_with_impl<F, const HAE: bool>(
        &mut self,
        new_len: usize,
        mut f: F,
    ) -> Result<(), ReservationError>
    where
        F: FnMut() -> T,
    {
        let p;
        let len;

        let cap = self.capacity.as_usize();
        let mut g = SetLenOnDrop::unarmed(self, 0);

        if mem::size_of::<T>() == 0 || cap <= C {
            len = cap;
            if new_len < len {
                g.sv.truncate(new_len);
                return Ok(());
            } else if new_len > len {
                if mem::size_of::<T>() != 0 && new_len > C {
                    g.sv.try_reserve_impl::<HAE>(new_len - len)?;
                    p = g.sv.buf.heap_mut_ptr();
                } else {
                    if mem::size_of::<T>() == 0 && new_len > L::MAX {
                        return Err(ReservationError::CapacityOverflow);
                    }
                    p = g.sv.buf.local_mut_ptr();
                }
            } else {
                return Ok(());
            }
        } else {
            len = g.sv.buf.heap_len().as_usize();
            if new_len < len {
                g.sv.truncate(new_len);
                return Ok(());
            } else if new_len > len {
                if new_len > cap {
                    g.sv.try_reserve_impl::<HAE>(new_len - len)?;
                }
                p = g.sv.buf.heap_mut_ptr();
            } else {
                return Ok(());
            }
        }

        g.len = len;
        g.armed = true;

        unsafe {
            let mut p = p.add(len);
            while g.len < new_len {
                p.write(f());
                p = p.add(1);
                g.len += 1;
            }
        }

        Ok(())
    }

    /// Creates a draining iterator that removes the specified range in the small-vector
    /// and yields the removed items.
    ///
    /// When the iterator is dropped, all elements in the range are removed from the small-vector,
    /// even if the iterator was not fully consumed.
    /// If the iterator is not dropped (with [`mem::forget`] for example),
    /// it is unspecified how many elements are removed.
    ///
    /// # Panics
    ///
    /// Panics if the starting point is greater than the end point or if the end point is greater
    /// than the length of the vector.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::small_vec;
    /// let mut v = small_vec![5; usize; 1, 2, 3, 4, 5];
    /// assert_eq!(v, [1, 2, 3, 4, 5]);
    /// for (index, i) in v.drain(0..3).enumerate() {
    ///     assert_eq!(index + 1, i);
    /// }
    /// assert_eq!(v, [4, 5]);
    /// ```
    ///
    /// [`mem::forget`]: core::mem::forget
    #[inline]
    pub fn drain<R>(&mut self, range: R) -> Drain<'_, T, L, SM, C>
    where
        R: RangeBounds<usize>,
    {
        let end = match range.end_bound() {
            Bound::Included(e) => e
                .checked_add(1)
                .unwrap_or_else(|| panic!("end bound overflows")),
            Bound::Excluded(e) => *e,
            Bound::Unbounded => self.len(),
        };

        let len = self.len();
        if end > len {
            panic!("invalid end bound");
        }

        let start = match range.start_bound() {
            Bound::Included(s) => *s,
            Bound::Excluded(s) => s
                .checked_add(1)
                .unwrap_or_else(|| panic!("start bound overflows")),
            Bound::Unbounded => 0,
        };

        if start > end {
            panic!("invalid range");
        }

        unsafe {
            let (iter, tail, tail_len) = if start < end {
                // set `len` to reflect the head only
                self.set_len(start);

                (
                    slice::from_raw_parts_mut(self.as_mut_ptr().add(start), end - start).iter_mut(),
                    L::new(end),
                    L::new(len - end),
                )
            } else {
                // empty drained range, mark it with an impossible combination of `tail/tail_len`
                ([].iter_mut(), L::new(L::MAX), L::new(L::MAX))
            };

            Drain {
                sv: ptr::NonNull::new_unchecked(self),
                iter,
                tail,
                tail_len,
            }
        }
    }
}

impl<T, const C: usize, L, SM> SmallVec<T, C, L, SM>
where
    T: Clone,
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    /// Resizes the small-vector in-place so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, the small-vector is extended by the difference,
    /// with each additional slot filled with `value`. If `new_len` is less than `len`,
    /// the small-vector is simply truncated.
    ///
    /// This method requires `T` to implement [`Clone`], in order to be able to clone the passed
    /// value. If you need more flexibility (or want to rely on [`Default`] instead of [`Clone`]),
    /// use [`resize_with`]. If you only need to resize to a smaller size, use [`truncate`].
    ///
    /// # Panics
    ///
    /// This method may panic on capacity reservation errors. See [`reserve`] for more information.
    ///
    /// See [`try_resize`] for a non-panic version of this method.
    ///
    /// [`resize_with`]: SmallVec::resize_with
    /// [`truncate`]: SmallVec::truncate
    /// [`reserve`]: SmallVec::reserve
    /// [`try_resize`]: SmallVec::try_resize
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::small_vec;
    /// let mut v = small_vec![4; u64; 1];
    /// v.resize(4, 0);
    /// assert_eq!(v, [1, 0, 0, 0]);
    /// v.resize(2, 3);
    /// assert_eq!(v, [1, 0]);
    /// ```
    #[inline]
    pub fn resize(&mut self, new_len: usize, value: T) {
        self.try_resize_impl::<DOHAE>(new_len, value)
            .expect("smallvec resize failed")
    }

    /// Tries to resize the small-vector in-place so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, the small-vector is extended by the difference,
    /// with each additional slot filled with `value`. If `new_len` is less than `len`,
    /// the small-vector is simply truncated.
    ///
    /// This method requires `T` to implement [`Clone`], in order to be able to clone the passed
    /// value. If you need more flexibility (or want to rely on [`Default`] instead of [`Clone`]),
    /// use [`try_resize_with`]. If you only need to resize to a smaller size, use [`truncate`].
    ///
    /// This is a non-panic version of [`resize`].
    ///
    /// [`try_resize_with`]: SmallVec::try_resize_with
    /// [`truncate`]: SmallVec::truncate
    /// [`resize`]: SmallVec::resize
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::{small_vec, mem::errors::ReservationError};
    /// # fn foo() -> Result<(), ReservationError> {
    /// let mut v = small_vec![2; usize];
    /// assert_eq!(v, []);
    /// v.try_resize(2, 7)?;
    /// assert_eq!(v, [7, 7]);
    /// v.try_resize(3, 5)?;
    /// assert_eq!(v, [7, 7, 5]);
    /// v.try_resize(1, 1)?;
    /// assert_eq!(v, [7]);
    /// # Ok(())
    /// # }
    /// # foo().unwrap();
    /// ```
    #[inline]
    pub fn try_resize(&mut self, new_len: usize, value: T) -> Result<(), ReservationError> {
        self.try_resize_impl::<NOHAE>(new_len, value)
    }

    #[inline]
    #[allow(clippy::comparison_chain)]
    fn try_resize_impl<const HAE: bool>(
        &mut self,
        new_len: usize,
        value: T,
    ) -> Result<(), ReservationError> {
        let p;
        let len;

        let cap = self.capacity.as_usize();
        let mut g = SetLenOnDrop::unarmed(self, 0);

        if mem::size_of::<T>() == 0 || cap <= C {
            len = cap;
            if new_len < len {
                g.sv.truncate(new_len);
                return Ok(());
            } else if new_len > len {
                if mem::size_of::<T>() != 0 && new_len > C {
                    g.sv.try_reserve_impl::<HAE>(new_len - len)?;
                    p = g.sv.buf.heap_mut_ptr();
                } else {
                    if mem::size_of::<T>() == 0 && new_len > L::MAX {
                        return Err(ReservationError::CapacityOverflow);
                    }
                    p = g.sv.buf.local_mut_ptr();
                }
            } else {
                return Ok(());
            }
        } else {
            len = g.sv.buf.heap_len().as_usize();
            if new_len < len {
                g.sv.truncate(new_len);
                return Ok(());
            } else if new_len > len {
                if new_len > cap {
                    g.sv.try_reserve_impl::<HAE>(new_len - len)?;
                }
                p = g.sv.buf.heap_mut_ptr();
            } else {
                return Ok(());
            }
        }

        g.len = len;
        g.armed = true;

        unsafe {
            let mut p = p.add(len);
            let to_add = new_len - len;

            for _ in 1..to_add {
                p.write(value.clone());
                p = p.add(1);
                g.len += 1;
            }

            // do not clone the last value
            if to_add > 0 {
                p.write(value);
                g.len += 1;
            }
        }
        Ok(())
    }
}

impl<T, const C: usize, L, SM> SmallVec<T, C, L, SM>
where
    T: Copy,
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    /// Extends the small-vector by copying elements from a slice.
    ///
    /// This is optimized for [`Copy`] types - elements are copied bytewise.
    ///
    /// # Panics
    ///
    /// This method panics on memory reservation errors. See [`reserve`] for more info.
    ///
    /// See [`try_copy_from_slice`] for a non-panic version of this method.
    ///
    /// [`reserve`]: SmallVec::reserve
    /// [`try_copy_from_slice`]: SmallVec::try_copy_from_slice
    ///
    /// # Examples
    /// ```rust
    /// # use cds::small_vec;
    /// let mut v = small_vec![5; 1, 2];
    /// assert_eq!(v, [1, 2]);
    /// v.copy_from_slice(&[3, 4]);
    /// assert_eq!(v, [1, 2, 3, 4]);
    /// ```
    #[inline]
    pub fn copy_from_slice(&mut self, s: &[T]) {
        self.try_copy_from_slice_impl::<DOHAE>(s)
            .expect("smallvec copy_from_slice failed")
    }

    /// Tries to extend the small-vector by copying elements from a slice.
    ///
    /// This is optimized for [`Copy`] types - elements are copied bytewise.
    ///
    /// This is a non-panic version of [`copy_from_slice`].
    ///
    /// [`copy_from_slice`]: SmallVec::copy_from_slice
    ///
    /// # Examples
    /// ```rust
    /// # use cds::{small_vec, mem::errors::ReservationError};
    /// # fn foo() -> Result<(), ReservationError> {
    /// let mut v = small_vec![5; 1, 2];
    /// assert_eq!(v, [1, 2]);
    /// v.try_copy_from_slice(&[3, 4])?;
    /// assert_eq!(v, [1, 2, 3, 4]);
    /// # Ok(())
    /// # }
    /// # foo().unwrap();
    /// ```
    #[inline]
    pub fn try_copy_from_slice(&mut self, s: &[T]) -> Result<(), ReservationError> {
        self.try_copy_from_slice_impl::<NOHAE>(s)
    }

    #[inline]
    fn try_copy_from_slice_impl<const HAE: bool>(
        &mut self,
        s: &[T],
    ) -> Result<(), ReservationError> {
        if mem::size_of::<T>() == 0 {
            if let Some(c) = self.capacity.checked_add_usize(s.len()) {
                self.capacity = c;
                Ok(())
            } else {
                Err(ReservationError::CapacityOverflow)
            }
        } else {
            let cur_len;
            let cap = self.capacity.as_usize();
            let (len, p) = if cap <= C {
                cur_len = cap;
                let cap = C;
                if cap - cur_len >= s.len() {
                    (&mut self.capacity, self.buf.local_mut_ptr())
                } else {
                    self.try_reserve_impl::<HAE>(s.len())?
                }
            } else {
                cur_len = self.buf.heap_len().as_usize();
                if cap - cur_len >= s.len() {
                    self.buf.heap_mut_len_mut_p()
                } else {
                    self.try_reserve_impl::<HAE>(s.len())?
                }
            };
            unsafe { ptr::copy_nonoverlapping(s.as_ptr(), p.add(cur_len), s.len()) };
            len.add_assign(s.len());
            Ok(())
        }
    }
}

#[inline]
unsafe fn clone_from_slice_unchecked<T, L>(s: &[T], len: &mut L, mut p: *mut T)
where
    T: Clone,
    L: LengthType,
{
    // Clone every element in source and append to the back of `self`.
    // Update `len` one-by-one, as `clone()` may panic and `self.drop()` may be implicitly
    // invoked. This way we drop only successfully written slots.
    for e in s {
        p.write(e.clone());
        p = p.add(1);
        len.add_assign(1);
    }
}

pub mod errors;
use errors::*;

mod drain;
pub use drain::*;

mod macros;
mod traits;

#[cfg(all(test, feature = "std"))]
mod test_smallvec;
