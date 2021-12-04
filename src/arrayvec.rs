//! A vector-like array.

use crate::{defs::SpareMemoryPolicy, errors::CapacityError};
use core::{marker::PhantomData, mem, ptr, result::Result, slice};

/// A continuous non-growable array with vector-like API.
///
/// Written as `ArrayVec<T, C>`, array vector has the capacity to store `C` elements of type `T`.
///
/// `ArrayVec` stores elements inline in the struct itself, and doesn't allocate memory on the heap.
///
/// The size of `ArrayVec` struct is not constant, as it depends on the requested capacity and
/// size of `T` (like a standard array).
///
/// `ArrayVec` may be created empty, with no elements.
/// However, once created, contrary to `Vec` which allocates memory only when elements are pushed,
/// `ArrayVec` occupies all the memory needed for the requested capacity.
///
/// The capacity of `ArrayVec` cannot be dynamically changed.
///
///
/// # Examples
///
/// ```rust
/// # use cds::{arrayvec::ArrayVec, array_vec, defs::Uninitialized};
/// let mut v = ArrayVec::<u64, Uninitialized, 12>::new();
/// assert_eq!(v.len(), 0);
/// assert_eq!(v.capacity(), 12);
/// assert_eq!(v.spare_capacity_len(), 12);
/// assert_eq!(v, []);
///
/// v.push(1);
/// v.push(2);
///
/// assert_eq!(v.len(), 2);
/// assert_eq!(v.capacity(), 12);
/// assert_eq!(v.spare_capacity_len(), 10);
/// assert_eq!(v, [1, 2]);
///
/// v[0] = 7;
/// assert_eq!(v, [7, 2]);
/// ```
///
/// The [`array_vec!`] macro is provided for convenient initialization:
///
/// ```rust
/// # use cds::array_vec;
/// let mut v = array_vec![12; u64; 1, 2, 3];
/// assert_eq!(v, [1, 2, 3]);
///
/// v.push(7);
/// assert_eq!(v, [1, 2, 3, 7]);
/// ```
///
/// [`push`] panics if there is no spare capacity:
///
/// ```should_panic
/// # use cds::{array_vec, errors::CapacityError};
/// let mut v = array_vec![3; 6, 7, 8];
/// assert_eq!(v.has_spare_capacity(), false);
/// v.push(9);  // <-- this panics as there is no spare capacity
/// ```
///
/// Avoid a panic with [`try_push`] method, which returns [`CapacityError`] instead:
///
/// ```rust
/// # use cds::{array_vec, errors::CapacityError};
/// let mut v = array_vec![3; 6, 7, 8];
/// assert!(matches!(v.try_push(9), Err(CapacityError)));
/// ```
///
/// An `ArrayVec` can be created from an iterator:
///
/// ```rust
/// # use cds::{arrayvec::ArrayVec, defs::Uninitialized};
/// type A = ArrayVec<u64, Uninitialized, 5>;
/// let vec = vec![1, 2, 3, 4, 5];
/// let a = vec.iter()
///            .map(|x| x * x)
///            .filter(|x| x % 2 == 0)
///            .collect::<A>();
/// assert_eq!(a, [4, 16]);
/// ```
///
/// If the iterator yields more than [`CAPACITY`] elements, the method panics:
///
/// ```should_panic
/// # use cds::{arrayvec::ArrayVec, defs::Uninitialized};
/// type A = ArrayVec<u64, Uninitialized, 3>; // <-- the capacity is 3
/// let vec = vec![1, 2, 3, 4, 5];
/// let a = vec.iter()                        // <-- but the iterator yields 5 elements
///            .map(|x| x * x)
///            .collect::<A>();               // <-- this panics
/// ```
///
/// Avoid a panic with [`try_from_iter`] method, which returns [`CapacityError`] instead:
///
/// ```rust
/// # use cds::{arrayvec::ArrayVec, errors::CapacityError, defs::Uninitialized};
/// type A = ArrayVec<u64, Uninitialized, 3>;
/// let vec = vec![1, 2, 3, 4, 5];
/// let iter = vec.iter().map(|x| x * x);
/// assert!(matches!(A::try_from_iter(iter), Err(CapacityError)));
/// ```
///
/// [`array_vec!`]: crate::array_vec
/// [`CAPACITY`]: ArrayVec::CAPACITY
/// [`try_from_iter`]: ArrayVec::try_from_iter
/// [`try_push`]: ArrayVec::try_push
/// [`push`]: ArrayVec::push
pub struct ArrayVec<T, SM, const C: usize>
where
    SM: SpareMemoryPolicy<T>,
{
    arr: [mem::MaybeUninit<T>; C],
    len: usize,
    phantom1: PhantomData<SM>,
}

impl<T, SM, const C: usize> ArrayVec<T, SM, C>
where
    SM: SpareMemoryPolicy<T>,
{
    /// The capacity of the array-vector as associated constant.
    ///
    /// The capacity can also be obtained via the [`capacity`] method.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::{arrayvec::ArrayVec, defs::Uninitialized};
    /// type A = ArrayVec<u64, Uninitialized, 8>;
    /// let v = A::new();
    /// assert_eq!(A::CAPACITY, 8);
    /// assert_eq!(v.capacity(), A::CAPACITY);
    /// ```
    ///
    /// [`capacity`]: ArrayVec::capacity
    pub const CAPACITY: usize = C;

    /// Creates an empty `ArrayVec`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::{arrayvec::ArrayVec, defs::Zeroed};
    /// let a = ArrayVec::<u64, Zeroed, 8>::new();
    /// assert_eq!(a.capacity(), 8);
    /// assert_eq!(a.len(), 0);
    /// ```
    #[inline]
    pub fn new() -> Self {
        let mut v = ArrayVec {
            // it is safe to call `assume_init` to create an array of `MaybeUninit`
            arr: unsafe { mem::MaybeUninit::uninit().assume_init() },
            len: 0,
            phantom1: PhantomData,
        };
        unsafe { SM::init(v.as_mut_ptr(), Self::CAPACITY) };
        v
    }

    /// Returns the number of elements in the array-vector.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::{arrayvec::ArrayVec, array_vec};
    /// let mut a = array_vec![12; 3, 4];
    /// assert_eq!(a.len(), 2);
    /// a.pop();
    /// assert_eq!(a.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the array-vector contains no elements.
    ///
    /// Equivalent to `len() == 0`.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::array_vec;
    /// assert_eq!(array_vec![3; u64].is_empty(), true);
    /// assert_eq!(array_vec![3; u64; 1].is_empty(), false);
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns `true` if the array-vector is completely full.
    ///
    /// Equivalent to `len() == capacity()`.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::array_vec;
    /// let mut v = array_vec![3; u64; 1, 2];
    /// assert_eq!(v.is_full(), false);
    /// v.push(3);
    /// assert_eq!(v.is_full(), true);
    /// ```
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len == Self::CAPACITY
    }

    /// Returns a raw pointer to the array-vector's buffer.
    ///
    /// The caller must ensure that the array-vector outlives the pointer this function returns.
    /// Otherwise, it will end up pointing to garbage.
    ///
    /// The caller must also ensure that the memory the pointer (non-transitively) points to is
    /// never written to (except inside an `UnsafeCell`) using this pointer or any pointer derived
    /// from it. If you need to mutate the contents of the slice, use [`as_mut_ptr`].
    ///
    /// [`as_mut_ptr`]: ArrayVec::as_mut_ptr
    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.arr.as_ptr() as *const T
    }

    /// Returns an unsafe mutable pointer to the array-vector's buffer.
    ///
    /// The caller must ensure that the array-vector outlives the pointer this function returns.
    /// Otherwise, it will end up pointing to garbage.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.arr.as_mut_ptr() as *mut T
    }

    /// Extracts a slice of the entire array-vector.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
    }

    /// Extracts a mutable slice of the entire array-vector.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
    }

    /// Returns the total number of elements the array-vector can hold.
    ///
    /// This is a convenience method. The capacity of the array-vector is known at compilation time
    /// and can be also obtained via the [`CAPACITY`] associated constant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::array_vec;
    /// let mut v = array_vec![17; u64];
    /// assert_eq!(v.capacity(), 17);
    /// ```
    ///
    /// [`CAPACITY`]: ArrayVec::CAPACITY
    #[inline]
    pub fn capacity(&self) -> usize {
        Self::CAPACITY
    }

    /// Returns the number of elements left until the array-vector is completely full.
    ///
    /// Equivalent to `capacity() - len()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::array_vec;
    /// let mut v = array_vec![2; u64];
    /// assert_eq!(v.capacity(), 2);
    /// assert_eq!(v.spare_capacity_len(), 2);
    ///
    /// v.push(1);
    /// assert_eq!(v.spare_capacity_len(), 1);
    /// ```
    #[inline]
    pub fn spare_capacity_len(&self) -> usize {
        Self::CAPACITY - self.len
    }

    /// Checks if there is spare capacity in the array-vector.
    ///
    /// Equivalent to `len() < capacity()`, `spare_capacity_len() != 0` and `!is_full()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::array_vec;
    /// let mut v = array_vec![2; u64];
    /// assert_eq!(v.capacity(), 2);
    /// assert_eq!(v.has_spare_capacity(), true);
    ///
    /// v.push(1);
    /// v.push(2);
    /// assert_eq!(v.has_spare_capacity(), false);
    /// ```
    #[inline]
    pub fn has_spare_capacity(&self) -> bool {
        self.len < Self::CAPACITY
    }

    /// Forces the length of the array-vector to `new_len`.
    ///
    /// # Safety
    ///
    /// - `new_len` must be less than or equal to the array-vector's [`CAPACITY`]
    /// - the elements at `old_len..new_len` must be initialized
    ///
    /// [`CAPACITY`]: ArrayVec::CAPACITY
    ///
    /// # Panics
    ///
    /// This method uses debug assertions to verify that `new_len` is in bounds.
    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= Self::CAPACITY);
        self.len = new_len;
    }

    /// Tries to append an element to the back of the array-vector.
    ///
    /// Returns [`CapacityError`] if there is no spare capacity to accommodate a new element.
    ///
    /// This is a non-panic version of [`push`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::{array_vec, errors::CapacityError};
    /// let mut v = array_vec![2; u64];
    /// assert!(v.try_push(1).is_ok());
    /// assert!(v.try_push(2).is_ok());
    /// assert!(matches!(v.try_push(3), Err(CapacityError)));
    /// assert_eq!(v, [1, 2]);
    /// ```
    ///
    /// [`push`]: ArrayVec::push
    #[inline]
    pub fn try_push(&mut self, e: T) -> Result<(), CapacityError> {
        if self.len < Self::CAPACITY {
            unsafe { self.push_unchecked(e) };
            Ok(())
        } else {
            Err(CapacityError {})
        }
    }

    /// Appends an element to the back of the array-vector.
    ///
    /// # Panics
    ///
    /// This method panics of there is no spare capacity to accommodate the new element.
    /// See [`try_push`] for a method that returns an error instead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::array_vec;
    /// let mut v = array_vec![2; u64];
    /// v.push(1);
    /// v.push(2);
    /// ```
    ///
    /// [`push`] panics if there is no spare capacity:
    ///
    /// ```should_panic
    /// # use cds::array_vec;
    /// let mut v = array_vec![2; u64];
    /// v.push(1);
    /// v.push(2);
    /// v.push(3);  // <-- this panics
    /// ```
    ///
    /// [`try_push`]: ArrayVec::try_push
    /// [`push`]: ArrayVec::push
    #[inline]
    pub fn push(&mut self, e: T) {
        self.try_push(e).expect("ArrayVec::push failed")
    }

    /// Removes the last element from an array-vector and returns it, or [`None`] if it is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::array_vec;
    /// let mut a = array_vec![3; 10];
    /// assert_eq!(a.pop(), Some(10));
    /// assert_eq!(a.pop(), None);
    /// ```
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.len > 0 {
            unsafe { Some(self.pop_unchecked()) }
        } else {
            None
        }
    }

    /// Removes the last element from array-vector and returns it without checking the length.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the array-vector is not empty.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::array_vec;
    /// let mut a = array_vec![3; 11, 12];
    /// if !a.is_empty() {
    ///     unsafe {
    ///         assert_eq!(a.pop_unchecked(), 12);
    ///     }
    /// }
    /// ```
    #[inline]
    pub unsafe fn pop_unchecked(&mut self) -> T {
        self.len -= 1;
        let p = self.as_mut_ptr().add(self.len);
        let e = p.read();
        SM::init(p, 1);
        e
    }

    /// Appends an element to the back of the array-vector without spare capacity check.
    ///
    /// This method is useful when spare capacity check is already done by the caller.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the array-vector has spare capacity to accommodate a new
    /// element.
    ///
    /// # Example
    /// ```rust
    /// # use cds::array_vec;
    /// let mut a = array_vec![3; usize; 1];
    /// while a.has_spare_capacity() {
    ///     unsafe { a.push_unchecked(0); }
    /// }
    /// assert_eq!(a, [1, 0, 0]);
    /// ```
    #[inline]
    pub unsafe fn push_unchecked(&mut self, e: T) {
        self.as_mut_ptr().add(self.len).write(e);
        self.len += 1;
    }

    /// Clears the array-vector, dropping all values.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::array_vec;
    /// let mut a = array_vec![16; 1, 2, 3];
    /// assert_eq!(a, [1, 2, 3]);
    /// a.clear();
    /// assert_eq!(a, []);
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.truncate(0)
    }

    /// Shortens the array-vector, keeping the first `len` elements and dropping the rest.
    ///
    /// If `len` is greater than array-vector's current length, this has no effect.
    ///
    /// # Safety
    ///
    /// Spare memory policy is invoked only if all truncated elements drop successfully. I.e, if
    /// any of the truncated elements panics during drop, spare memory policy isn't invoked
    /// at all, including on successfully dropped elements.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::array_vec;
    /// let mut a = array_vec![8; 1, 2, 3];
    /// assert_eq!(a, [1, 2, 3]);
    /// a.truncate(1);
    /// assert_eq!(a, [1]);
    /// a.truncate(2);
    /// assert_eq!(a, [1]);
    /// ```
    pub fn truncate(&mut self, len: usize) {
        if len < self.len {
            unsafe {
                // create a slice of truncated slots
                let s = slice::from_raw_parts_mut(self.as_mut_ptr().add(len), self.len - len);

                // `drop` of any of the truncated slots may panic, which may trigger destruction
                // of `self`. Thus, update `self.len` *before* calling `drop_in_place` to avoid
                // a possible double-drop of a truncated slot.
                self.len = len;

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

    /// Returns an iterator over the slice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::array_vec;
    /// let v = array_vec![3; 1, 2];
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
    /// # use cds::array_vec;
    /// let mut v = array_vec![3; 1, 2];
    /// for e in v.iter_mut() {
    ///     *e *= 2;
    /// }
    /// assert_eq!(v, [2, 4]);
    /// ```
    #[inline]
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, T> {
        self.as_mut_slice().iter_mut()
    }

    /// Creates an array-vector from an iterator.
    ///
    /// Returns [`CapacityError`] if the iterator yields more than [`CAPACITY`] elements.
    ///
    /// [`CAPACITY`]: ArrayVec::CAPACITY
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::{arrayvec::ArrayVec, errors::CapacityError, defs::Uninitialized};
    /// # use std::error::Error;
    /// # fn example() -> Result<(), CapacityError> {
    /// type A = ArrayVec<usize, Uninitialized, 3>;
    /// let a = [1, 2, 3];
    /// let v = A::try_from_iter(a.iter().filter(|x| **x % 2 == 0).cloned())?;
    /// assert_eq!(v, [2]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_from_iter<I>(iter: I) -> Result<Self, CapacityError>
    where
        I: IntoIterator<Item = T>,
    {
        let mut tmp = Self::new();
        let mut p = tmp.as_mut_ptr();

        for e in iter {
            if tmp.len >= Self::CAPACITY {
                return Err(CapacityError {});
            }
            unsafe {
                p.write(e);
                p = p.add(1);
                tmp.len += 1;
            }
        }

        Ok(tmp)
    }

    /// Inserts an element at position `index` within the vector, shifting all elements after it to
    /// the right.
    ///
    /// Note that the worst case performance of this operation is O(n), because all elements of the
    /// array may be shifted right. If order of elements is not needed to be preserved, use [`push`]
    /// instead.
    ///
    /// # Panics
    ///
    /// This method panics if any of the following conditions is met:
    /// - `index > len()`
    /// - there is no spare capacity in the array-vector
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::array_vec;
    /// let mut v = array_vec![3; u64; 1, 2];
    /// assert_eq!(v, [1, 2]);
    /// v.insert(1, 0);
    /// assert_eq!(v, [1, 0, 2]);
    /// ```
    /// [`insert`] panics if `index > len()`:
    ///
    /// ```should_panic
    /// # use cds::array_vec;
    /// let mut v = array_vec![3; u64; 1];
    /// assert_eq!(v.len(), 1);
    /// v.insert(2, 1);  // <-- this panics because 2 > v.len()
    /// ```
    ///
    /// [`insert`] also panics if there is no spare capacity:
    ///
    /// ```should_panic
    /// # use cds::array_vec;
    /// let mut v = array_vec![2; u64; 1, 2];
    /// assert_eq!(v.has_spare_capacity(), false);
    /// v.insert(0, 0);  // <-- this panics
    /// ```
    ///
    /// [`insert`]: ArrayVec::insert
    /// [`push`]: ArrayVec::push
    #[inline]
    pub fn insert(&mut self, index: usize, element: T) {
        self.try_insert(index, element).expect("cannot insert")
    }

    /// Tries to insert an element at position `index` within the vector, shifting all elements
    /// after it to the right.
    ///
    /// Returns [`CapacityError`] if there is no spare capacity in the array vector.
    ///
    /// Note that the worst case performance of this operation is O(n), because all elements of the
    /// array may be shifted right. If order of elements is not needed to be preserved,
    /// use [`try_push`] instead.
    ///
    /// This is a non-panic version of [`insert`].
    ///
    /// # Panics
    ///
    /// This method panics if `index > len()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::{array_vec, errors::CapacityError};
    /// let mut v = array_vec![3; u64; 1, 2];
    /// assert!(v.try_insert(0, 0).is_ok());
    /// assert_eq!(v, [0, 1, 2]);
    /// assert!(matches!(v.try_insert(1, 3), Err(CapacityError)));
    /// ```
    ///
    /// [`try_push`]: ArrayVec::try_push
    /// [`insert`]: ArrayVec::insert
    #[inline]
    pub fn try_insert(&mut self, index: usize, element: T) -> Result<(), CapacityError> {
        if self.len >= Self::CAPACITY {
            return Err(CapacityError {});
        }
        if index > self.len {
            panic!("index is out of bounds [0, {}]: {}", self.len, index);
        }
        unsafe {
            self.insert_unchecked(index, element);
        }
        Ok(())
    }

    /// Inserts an element at position `index` within the vector, shifting all elements after it
    /// to the right.
    ///
    /// # Safety
    ///
    /// The caller must ensure the following conditions:
    /// - `index <= len()`
    /// - there is spare capacity in the array-vector
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::array_vec;
    /// let mut v = array_vec![3; u64; 1, 2];
    /// assert_eq!(v, [1, 2]);
    /// assert_eq!(v.has_spare_capacity(), true);
    ///
    /// unsafe { v.insert_unchecked(0, 0) };
    /// assert_eq!(v, [0, 1, 2]);
    ///
    /// v.pop();
    /// assert_eq!(v, [0, 1]);
    /// assert_eq!(v.has_spare_capacity(), true);
    ///
    /// unsafe { v.insert_unchecked(2, 2) };
    /// assert_eq!(v, [0, 1, 2]);
    /// ```
    pub unsafe fn insert_unchecked(&mut self, index: usize, element: T) {
        let p = self.as_mut_ptr().add(index);
        ptr::copy(p, p.add(1), self.len - index);
        p.write(element);
        self.len += 1;
    }

    /// Removes and returns the element at position `index` within the vector, shifting all elements
    /// after it to the left.
    ///
    /// Note: Because this shifts over the remaining elements, it has a worst-case performance of
    /// O(n). If you don’t need the order of elements to be preserved, use [`swap_remove`] instead.
    ///
    /// # Panics
    ///
    /// This method panics if `index >= len()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::array_vec;
    /// let mut v = array_vec![3; u64; 1, 2, 3];
    /// assert_eq!(v, [1, 2, 3]);
    /// assert_eq!(v.remove(1), 2);
    /// assert_eq!(v, [1, 3]);
    /// ```
    ///
    /// [`remove`] panics if `index` is out of bounds:
    ///
    /// ```should_panic
    /// # use cds::array_vec;
    /// let mut v = array_vec![2; u64; 1];
    /// assert_eq!(v, [1]);
    /// v.remove(1);  // <-- this panics because index=1 is out of bounds
    /// ```
    ///
    /// [`swap_remove`]: ArrayVec::swap_remove
    /// [`remove`]: ArrayVec::remove
    #[inline]
    pub fn remove(&mut self, index: usize) -> T {
        if index >= self.len {
            panic!("index is out of bounds [0, {}): {}", self.len, index);
        }
        unsafe { self.remove_unchecked(index) }
    }

    /// Tries to remove and return the element at position `index` within the vector, shifting all
    /// elements after it to the left.
    ///
    /// Returns `None` if `index` is out of bounds.
    ///
    /// Note: Because this shifts over the remaining elements, it has a worst-case performance of
    /// O(n). If you don’t need the order of elements to be preserved, use [`try_swap_remove`]
    /// instead.
    ///
    /// This is a non-panic version of [`remove`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::array_vec;
    /// let mut v = array_vec![3; u64; 1, 2, 3];
    /// assert_eq!(v.try_remove(3), None);
    /// assert_eq!(v.try_remove(0), Some(1));
    /// assert_eq!(v, [2, 3]);
    /// ```
    ///
    /// [`remove`]: ArrayVec::remove
    /// [`try_swap_remove`]: ArrayVec::try_swap_remove
    #[inline]
    pub fn try_remove(&mut self, index: usize) -> Option<T> {
        if index < self.len {
            unsafe { Some(self.remove_unchecked(index)) }
        } else {
            None
        }
    }

    /// Removes and returns the element at position `index` within the array-vector, shifting all
    /// elements after it to the left.
    ///
    /// Note: Because this shifts over the remaining elements, it has a worst-case performance of
    /// O(n). If you don’t need the order of elements to be preserved, use [`swap_remove_unchecked`]
    /// instead.
    ///
    /// This is the unchecked version of [`remove`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that `index < len()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::array_vec;
    /// let mut v = array_vec![3; u64; 1, 2, 3];
    /// assert_eq!(v, [1, 2, 3]);
    /// assert_eq!(unsafe { v.remove_unchecked(0) }, 1);
    /// assert_eq!(v, [2, 3]);
    /// ```
    ///
    /// [`remove`]: ArrayVec::remove
    /// [`swap_remove_unchecked`]: ArrayVec::swap_remove_unchecked
    pub unsafe fn remove_unchecked(&mut self, index: usize) -> T {
        let base = self.as_mut_ptr();
        let p = base.add(index);
        let tmp = p.read();
        ptr::copy(p.add(1), p, self.len - index - 1);
        self.len -= 1;
        SM::init(base.add(self.len), 1);
        tmp
    }

    /// Removes an element at position `index` from the array-vector and returns it.
    ///
    /// The removed element is replaced by the last element of the array-vector.
    ///
    /// This does not preserve ordering, but is O(1).
    ///
    /// # Panics
    ///
    /// This method panics of `index` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::array_vec;
    /// let mut v = array_vec![3; u64; 1, 2, 3];
    /// assert_eq!(v.swap_remove(0), 1);
    /// assert_eq!(v, [3, 2]);
    /// ```
    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> T {
        if index >= self.len {
            panic!("index is out of bounds [0, {}): {}", self.len, index);
        }
        unsafe { self.swap_remove_unchecked(index) }
    }

    /// Tries to remove an element at position `index` from the array-vector and returns it.
    ///
    /// The removed element is replaced by the last element of the array-vector.
    ///
    /// This does not preserve ordering, but is O(1).
    ///
    /// Returns `None` if `index` is out of bounds.
    ///
    /// This is a non-panic version of [`swap_remove`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::array_vec;
    /// let mut v = array_vec![3; u64; 1, 2, 3];
    /// assert_eq!(v.try_swap_remove(3), None);
    /// assert_eq!(v.try_swap_remove(0), Some(1));
    /// assert_eq!(v, [3, 2]);
    /// ```
    ///
    /// [`swap_remove`]: ArrayVec::swap_remove
    #[inline]
    pub fn try_swap_remove(&mut self, index: usize) -> Option<T> {
        if index < self.len {
            unsafe { Some(self.swap_remove_unchecked(index)) }
        } else {
            None
        }
    }

    /// Removes an element at position `index` from the array-vector and returns it, without
    /// bounds check.
    ///
    /// The removed element is replaced by the last element of the array-vector.
    /// This does not preserve ordering, but is O(1).
    ///
    /// # Safety
    ///
    /// The caller must ensure that `index < len()`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use cds::array_vec;
    /// let mut v = array_vec![3; u64; 1, 2, 3];
    /// assert_eq!(v, [1, 2, 3]);
    /// assert_eq!(unsafe { v.swap_remove_unchecked(0) }, 1);
    /// assert_eq!(v, [3, 2]);
    /// ```
    pub unsafe fn swap_remove_unchecked(&mut self, index: usize) -> T {
        let base = self.as_mut_ptr();
        let p = base.add(index);
        let tmp = p.read();
        self.len -= 1;
        let last = base.add(self.len);
        if index < self.len {
            ptr::copy(last, p, 1);
        }
        SM::init(last, 1);
        tmp
    }
}

impl<T, SM, const C: usize> ArrayVec<T, SM, C>
where
    T: Clone,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn _clone_from(&mut self, other: &Self) {
        unsafe {
            self._clone_from_unchecked(other);
        }
    }

    #[inline]
    unsafe fn _clone_from_unchecked(&mut self, s: &[T]) {
        debug_assert!(self.is_empty());
        let mut p = self.as_mut_ptr();
        // Clone every element in source and append to the back of `self`.
        // Update `len` one-by-one, as `clone()` may panic and `self.drop()` may be implicitly
        // invoked. This way we drop only successfully written slots.
        for e in s {
            p.write(e.clone());
            p = p.add(1);
            self.len += 1;
        }
    }
}

mod macros;
mod traits;

#[cfg(test)]
mod test_arrayvec;
