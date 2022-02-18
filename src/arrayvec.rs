//! A vector-like array.

use crate::{
    errors::{CapacityError, CapacityErrorVal, InsertError, InsertErrorVal},
    len::LengthType,
    mem::SpareMemoryPolicy,
};
use core::{
    marker::PhantomData,
    mem,
    ops::{Bound, RangeBounds},
    ptr,
    result::Result,
    slice,
};

mod drain;
pub use drain::*;

mod retain;
use retain::*;

/// A continuous non-growable array with vector-like API.
///
/// Written as `ArrayVec<T, L, SM, C>`, array vector has the capacity to store `C` elements of type
/// `T`.
///
/// It uses type `L` as [`length type`], and `SM` as [`spare memory policy`].
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
/// [`spare memory policy`]: SpareMemoryPolicy
/// [`length type`]: LengthType
///
///
/// # Examples
///
/// ```rust
/// # use cds::{arrayvec::ArrayVec, array_vec, len::U8, mem::Uninitialized};
/// let mut v = ArrayVec::<u64, U8, Uninitialized, 12>::new();
/// assert_eq!(v.len(), 0);
/// assert_eq!(v.capacity(), 12);
/// assert_eq!(v.spare_capacity(), 12);
/// assert_eq!(v, []);
///
/// v.push(1);
/// v.push(2);
///
/// assert_eq!(v.len(), 2);
/// assert_eq!(v.capacity(), 12);
/// assert_eq!(v.spare_capacity(), 10);
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
/// # use cds::{arrayvec::ArrayVec, len::U8, mem::Uninitialized};
/// type A = ArrayVec<u64, U8, Uninitialized, 5>;
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
/// # use cds::{arrayvec::ArrayVec, len::U64, mem::Uninitialized};
/// type A = ArrayVec<u64, U64, Uninitialized, 3>; // <-- the capacity is 3
/// let vec = vec![1, 2, 3, 4, 5];
/// let a = vec.iter()                             // <-- but the iterator yields 5 elements
///            .map(|x| x * x)
///            .collect::<A>();                    // <-- this panics
/// ```
///
/// Avoid a panic with [`try_from_iter`] method, which returns [`CapacityError`] instead:
///
/// ```rust
/// # use cds::{arrayvec::ArrayVec, errors::CapacityError, len::U64, mem::Uninitialized};
/// type A = ArrayVec<u64, U64, Uninitialized, 3>;
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
pub struct ArrayVec<T, L, SM, const C: usize>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    arr: [mem::MaybeUninit<T>; C],
    len: L,
    phantom1: PhantomData<SM>,
}

impl<T, L, SM, const C: usize> ArrayVec<T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    /// The capacity of the array-vector as associated constant.
    ///
    /// The capacity can also be obtained via the [`capacity`] method.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::{arrayvec::ArrayVec, len::U8, mem::Uninitialized};
    /// type A = ArrayVec<u64, U8, Uninitialized, 8>;
    /// let v = A::new();
    /// assert_eq!(A::CAPACITY, 8);
    /// assert_eq!(v.capacity(), A::CAPACITY);
    /// ```
    ///
    /// [`capacity`]: ArrayVec::capacity
    pub const CAPACITY: usize = C;

    /// Creates an empty `ArrayVec`.
    ///
    /// # Safety
    ///
    /// This method panics if requested capacity `C` exceeds the maximal value that can be stored in
    /// length type `L`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::{arrayvec::ArrayVec, len::U8, mem::Zeroed};
    /// let a = ArrayVec::<u64, U8, Zeroed, 8>::new();
    /// assert_eq!(a.capacity(), 8);
    /// assert_eq!(a.len(), 0);
    /// ```
    #[inline]
    pub fn new() -> Self {
        assert!(C <= L::MAX);
        let mut v = ArrayVec {
            // it is safe to call `assume_init` to create an array of `MaybeUninit`
            arr: unsafe { mem::MaybeUninit::uninit().assume_init() },
            len: L::new(0),
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
        self.len.as_usize()
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
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len.as_usize()) }
    }

    /// Extracts a mutable slice of the entire array-vector.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len.as_usize()) }
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

    /// Returns the number of elements the array-vector can hold in addition to already held ones.
    ///
    /// Equivalent to `capacity() - len()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::array_vec;
    /// let mut v = array_vec![2; u64];
    /// assert_eq!(v.capacity(), 2);
    /// assert_eq!(v.spare_capacity(), 2);
    ///
    /// v.push(1);
    /// assert_eq!(v.capacity(), 2);
    /// assert_eq!(v.spare_capacity(), 1);
    /// ```
    #[inline]
    pub fn spare_capacity(&self) -> usize {
        Self::CAPACITY - self.len.as_usize()
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
    /// Note that this method doesn't [`drop`] elements, which may lead to a resource leak if
    /// `new_len < len()` and `T` has a custom [`Drop`] implementation. See [`truncate`] for a
    /// method that handles array-vector truncation properly.
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
    ///
    /// [`drop`]: core::mem::drop
    /// [`Drop`]: core::ops::Drop
    /// [`truncate`]: ArrayVec::truncate
    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= Self::CAPACITY);
        self.len.set(new_len);
    }

    /// Appends an element to the back of the array-vector.
    ///
    /// # Panics
    ///
    /// This method panics if there is no spare capacity to accommodate the new element.
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
        if self.len >= Self::CAPACITY {
            panic!("insufficient capacity");
        }
        unsafe { self.push_unchecked(e) };
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

    /// Tries to append an element to the back of the array-vector.
    ///
    /// Returns [`CapacityErrorVal`] if there is no spare capacity to accommodate a new element.
    ///
    /// The difference between this method and [`try_push`] is that in case of an error
    /// [`try_push_val`] returns the element back to the caller.
    ///
    /// This is a non-panic version of [`push`].
    ///
    /// # Examples
    /// ```rust
    /// # use cds::{array_vec, errors::CapacityErrorVal};
    /// let mut v = array_vec![2; u64];
    /// assert_eq!(v, []);
    ///
    /// assert!(v.try_push_val(1).is_ok());
    /// assert!(v.try_push_val(2).is_ok());
    /// assert_eq!(v, [1, 2]);
    ///
    /// assert!(matches!(v.try_push_val(3), Err(CapacityErrorVal(e)) if e == 3));
    /// assert_eq!(v, [1, 2]);
    /// ```
    ///
    /// [`try_push_val`]: ArrayVec::try_push_val
    /// [`try_push`]: ArrayVec::try_push
    /// [`push`]: ArrayVec::push
    #[inline]
    pub fn try_push_val(&mut self, value: T) -> Result<(), CapacityErrorVal<T>> {
        if self.len < Self::CAPACITY {
            unsafe { self.push_unchecked(value) };
            Ok(())
        } else {
            Err(CapacityErrorVal(value))
        }
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
    pub unsafe fn push_unchecked(&mut self, value: T) {
        let len = self.len();
        self.as_mut_ptr().add(len).write(value);
        self.set_len(len + 1);
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
        let p = self.as_mut_ptr().add(self.len.as_usize());
        let e = p.read();
        SM::init(p, 1);
        e
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
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        let my_len = self.len.as_usize();

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
    /// # use cds::{arrayvec::ArrayVec, errors::CapacityError, len::U8, mem::Uninitialized};
    /// # use std::error::Error;
    /// # fn example() -> Result<(), CapacityError> {
    /// type A = ArrayVec<usize, U8, Uninitialized, 3>;
    /// let a = [1, 2, 3];
    /// let v = A::try_from_iter(a.iter().filter(|x| **x % 2 == 0).cloned())?;
    /// assert_eq!(v, [2]);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
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
    /// Returns [`InsertError`] if there is no spare capacity in the array vector, or if `index` is
    /// out of bounds.
    ///
    /// Note that in case of an error `value` is lost if it is not [`Copy`]. Use [`try_insert_val`]
    /// to receive the element back in case of an error.
    ///
    /// Note that the worst case performance of this operation is O(n), because all elements of the
    /// array may be shifted right. If order of elements is not needed to be preserved,
    /// use [`try_push`] instead.
    ///
    /// This is a non-panic version of [`insert`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::{array_vec, errors::InsertError};
    /// let mut v = array_vec![3; u64; 1, 2];
    /// assert!(matches!(v.try_insert(3, 3), Err(InsertError::InvalidIndex)));
    ///
    /// assert!(v.try_insert(0, 0).is_ok());
    /// assert_eq!(v, [0, 1, 2]);
    /// assert!(matches!(v.try_insert(1, 3), Err(InsertError::InsufficientCapacity)));
    /// ```
    ///
    /// [`try_push`]: ArrayVec::try_push
    /// [`try_insert_val`]: ArrayVec::try_insert_val
    /// [`insert`]: ArrayVec::insert
    #[inline]
    pub fn try_insert(&mut self, index: usize, value: T) -> Result<(), InsertError> {
        let len = self.len();
        if index > len {
            return Err(InsertError::InvalidIndex);
        }
        if len >= Self::CAPACITY {
            return Err(InsertError::InsufficientCapacity);
        }
        unsafe {
            self.insert_unchecked(index, value);
        }
        Ok(())
    }

    /// Tries to insert an element at position `index` within the array-vector,
    /// shifting all elements after it to the right.
    ///
    /// Returns [`InsertErrorVal`] if there is no spare capacity in the array vector,
    /// or if `index` is out of bounds.
    ///
    /// The difference between this method and [`try_insert`], is that in case of an error
    /// [`try_insert_val`] returns the element back to the caller.
    ///
    /// Note that the worst case performance of this operation is O(n), because all elements of the
    /// array may be shifted right. If order of elements is not needed to be preserved,
    /// use [`try_push_val`] instead.
    ///
    /// This is a non-panic version of [`insert`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::{array_vec, errors::InsertErrorVal};
    /// let mut v = array_vec![3; u64; 1, 2];
    /// assert!(matches!(
    ///     v.try_insert_val(5, 3),
    ///     Err(InsertErrorVal::InvalidIndex(v)) if v == 3
    /// ));
    /// assert_eq!(v, [1, 2]);
    ///
    /// assert!(v.try_insert_val(0, 0).is_ok());
    /// assert_eq!(v, [0, 1, 2]);
    ///
    /// assert!(matches!(
    ///     v.try_insert_val(1, 5),
    ///     Err(InsertErrorVal::InsufficientCapacity(v)) if v == 5
    /// ));
    /// assert_eq!(v, [0, 1, 2]);
    /// ```
    ///
    /// [`try_insert_val`]: ArrayVec::try_insert_val
    /// [`try_insert`]: ArrayVec::try_insert
    /// [`try_push_val`]: ArrayVec::try_push_val
    /// [`insert`]: ArrayVec::insert
    #[inline]
    pub fn try_insert_val(&mut self, index: usize, value: T) -> Result<(), InsertErrorVal<T>> {
        if index > self.len.as_usize() {
            return Err(InsertErrorVal::InvalidIndex(value));
        }
        if self.len >= Self::CAPACITY {
            return Err(InsertErrorVal::InsufficientCapacity(value));
        }
        unsafe {
            self.insert_unchecked(index, value);
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
    #[inline]
    pub unsafe fn insert_unchecked(&mut self, index: usize, value: T) {
        let len = self.len();
        let p = self.as_mut_ptr().add(index);
        ptr::copy(p, p.add(1), len - index);
        p.write(value);
        self.set_len(len + 1);
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
        if index >= self.len.as_usize() {
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
        if index < self.len.as_usize() {
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
    #[inline]
    pub unsafe fn remove_unchecked(&mut self, index: usize) -> T {
        let base = self.as_mut_ptr();
        let p = base.add(index);
        let tmp = p.read();
        ptr::copy(p.add(1), p, self.len.as_usize() - index - 1);
        self.len -= 1;
        SM::init(base.add(self.len.as_usize()), 1);
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
    /// let mut v = array_vec![4; u64; 1, 2, 3, 4];
    /// assert_eq!(v.swap_remove(1), 2);
    /// assert_eq!(v, [1, 4, 3]);
    /// ```
    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> T {
        let len = self.len();
        if index >= len {
            panic!("index is out of bounds [0, {}): {}", len, index);
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
        if index < self.len.as_usize() {
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
    /// let mut v = array_vec![4; u64; 1, 2, 3, 4];
    /// assert_eq!(v, [1, 2, 3, 4]);
    /// assert_eq!(unsafe { v.swap_remove_unchecked(2) }, 3);
    /// assert_eq!(v, [1, 2, 4]);
    /// ```
    #[inline]
    pub unsafe fn swap_remove_unchecked(&mut self, index: usize) -> T {
        let base = self.as_mut_ptr();
        let p = base.add(index);
        let value = p.read();
        self.len -= 1;
        let last = base.add(self.len.as_usize());
        ptr::copy(last, p, 1);
        SM::init(last, 1);
        value
    }

    /// Creates a draining iterator that removes the specified range in the array-vector
    /// and yields the removed items.
    ///
    /// When the iterator is dropped, all elements in the range are removed from the array-vector,
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
    /// # use cds::array_vec;
    /// let mut a = array_vec![5; usize; 1, 2, 3, 4, 5];
    /// assert_eq!(a, [1, 2, 3, 4, 5]);
    /// for (index, i) in a.drain(0..3).enumerate() {
    ///     assert_eq!(index + 1, i);
    /// }
    /// assert_eq!(a, [4, 5]);
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

        if end > self.len() {
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
            let len = self.len();
            let (iter, tail, tail_len) = if start < end {
                // set `len` to reflect the head only
                self.set_len(start);

                (
                    slice::from_raw_parts(self.as_ptr().add(start), end - start).iter(),
                    L::new(end),
                    L::new(len - end),
                )
            } else {
                // empty drained range, mark it with an impossible combination of `tail/tail_len`
                ((&[]).iter(), L::new(L::MAX), L::new(L::MAX))
            };

            Drain {
                av: ptr::NonNull::new_unchecked(self),
                iter,
                tail,
                tail_len,
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
    /// # use cds::array_vec;
    /// let mut a = array_vec![5; 0, 1, 2, 3, 4];
    /// assert_eq!(a, [0, 1, 2, 3, 4]);
    /// a.retain(|e| (*e & 1) != 0);
    /// assert_eq!(a, [1, 3]);
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
    /// # use cds::array_vec;
    /// let mut a = array_vec![5; 0, 1, 2, 3, 4];
    /// assert_eq!(a, [0, 1, 2, 3, 4]);
    /// a.retain_mut(|e| if (*e & 1) == 0 {
    ///    *e *= *e;
    ///    true
    /// } else {
    ///    false
    /// });
    /// assert_eq!(a, [0, 4, 16]);
    /// ```
    #[inline]
    pub fn retain_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        let len = self.len();

        // set `len` to zero, to avoid double-drop of deleted items.
        // `len` is restored by RetainGuard.
        unsafe { self.set_len(0) };

        let mut g = RetainGuard {
            av: self,
            len,
            deleted: 0,
            processed: 0,
        };

        unsafe {
            // no empty slot found yet, so there is nothing to move
            while g.processed < len {
                let item_mut_ref = &mut *g.av.as_mut_ptr().add(g.processed);
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
                let item_mut_ref = &mut *g.av.as_mut_ptr().add(g.processed);
                if !f(item_mut_ref) {
                    // update counters before drop_in_place, as it may panic
                    g.processed += 1;
                    g.deleted += 1;
                    ptr::drop_in_place(item_mut_ref);
                    continue;
                } else {
                    ptr::copy_nonoverlapping(
                        item_mut_ref as *const _,
                        g.av.as_mut_ptr().add(g.processed - g.deleted),
                        1,
                    );
                }
                g.processed += 1;
            }
        }
    }

    /// Returns the remaining spare capacity of the array-vector as a slice of `MaybeUninit<T>`.
    ///
    /// The returned slice can be used to fill the array-vector with data (e.g. by reading from a
    /// file) before marking the data as initialized using the [`set_len`] method.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::array_vec;
    /// let mut a = array_vec![32; 1, 2];   // <-- an array-vector for IO of 32 elements
    /// assert_eq!(a, [1, 2]);
    ///
    /// let spare_capacity = a.spare_capacity_mut();
    /// spare_capacity[0].write(3);         // <-- read another 2 elements into the array-vector
    /// spare_capacity[1].write(4);
    ///
    /// unsafe { a.set_len(a.len() + 2) };  // <-- reflect the new size
    ///
    /// assert_eq!(a, [1, 2, 3, 4]);
    /// ```
    ///
    /// [`set_len`]: ArrayVec::set_len
    #[inline]
    pub fn spare_capacity_mut(&mut self) -> &mut [mem::MaybeUninit<T>] {
        unsafe {
            slice::from_raw_parts_mut(self.arr.as_mut_ptr().add(self.len()), self.spare_capacity())
        }
    }

    /// Returns array-vector content as a slice of `T`, along with the remaining spare capacity of
    /// the array-vector as a slice of `MaybeUninit<T>`.
    ///
    /// The returned spare capacity slice can be used to fill the array-vector with data
    /// (e.g. by reading from a file) before marking the data as initialized using the [`set_len`]
    /// method.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::array_vec;
    /// let mut a = array_vec![32; 1, 2];   // <-- an array-vector for IO of 32 elements
    ///
    /// let (init, spare) = a.split_at_spare_mut();
    /// assert_eq!(init, &[1, 2]);
    ///
    /// assert_eq!(spare.len(), 30);        // <-- read another 2 elements into the array-vector
    /// spare[0].write(3);
    /// spare[1].write(4);
    ///
    /// unsafe { a.set_len(a.len() + 2) };  // <-- reflect the new size
    ///
    /// assert_eq!(a, [1, 2, 3, 4]);
    /// ```
    ///
    /// [`set_len`]: ArrayVec::set_len
    #[inline]
    pub fn split_at_spare_mut(&mut self) -> (&mut [T], &mut [mem::MaybeUninit<T>]) {
        let len = self.len();
        let spare_capacity = self.spare_capacity();
        let p = self.as_mut_ptr();

        unsafe {
            (
                slice::from_raw_parts_mut(p, len),
                slice::from_raw_parts_mut(p.add(len) as *mut mem::MaybeUninit<T>, spare_capacity),
            )
        }
    }

    /// Resizes the array-vector in-place so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, the array-vector is extended by the difference,
    /// with each additional slot filled with the result of calling the closure `f`.
    /// The return values from `f` will end up in the array-vector in the order they have been
    /// generated.
    ///
    /// If `new_len` is less than `len`, the array-vector is simply truncated.
    ///
    /// This method uses a closure to create new values on every push.
    /// If you’d rather [`Clone`] a given value, use [`try_resize`].
    /// If you want to use the [`Default`] trait to generate values, you can pass
    /// [`Default::default`] as the second argument.
    ///
    /// # Panics
    ///
    /// This method panics of `new_len > CAPACITY`. To avoid panic use [`try_resize_with`] which
    /// returns [`CapacityError`] instead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::array_vec;
    /// let mut a = array_vec![5; 1];
    /// assert_eq!(a, [1]);
    ///
    /// let mut g = 1;
    ///
    /// a.resize_with(3, || { g += 1; g });
    /// assert_eq!(a, [1, 2, 3]);
    ///
    /// a.resize_with(5, || { g *= 2; g });
    /// assert_eq!(a, [1, 2, 3, 6, 12]);
    ///
    /// a.resize_with(1, || 0);
    /// assert_eq!(a, [1]);
    /// ```
    ///
    /// [`try_resize`]: ArrayVec::try_resize
    /// [`try_resize_with`]: ArrayVec::try_resize_with
    /// [`resize_with`]: ArrayVec::resize_with
    /// [`Clone`]: core::clone::Clone
    /// [`Default`]: core::default::Default
    /// [`Default::default`]: core::default::Default::default
    #[inline]
    pub fn resize_with<F>(&mut self, new_len: usize, f: F)
    where
        F: FnMut() -> T,
    {
        self.try_resize_with(new_len, f)
            .expect("insufficient capacity")
    }

    /// Tries to resize the array-vector in-place so that `len` is equal to `new_len`.
    ///
    /// This is a non-panic version of [`resize_with`].
    ///
    /// If `new_len` is greater than `len`, the array-vector is extended by the difference,
    /// with each additional slot filled with the result of calling the closure `f`.
    /// The return values from `f` will end up in the array-vector in the order they have been
    /// generated.
    ///
    /// If `new_len` is less than `len`, the array-vector is simply truncated.
    ///
    /// This method uses a closure to create new values on every push.
    /// If you’d rather [`Clone`] a given value, use [`try_resize`].
    /// If you want to use the [`Default`] trait to generate values, you can pass
    /// [`Default::default`] as the second argument.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::{array_vec, errors::CapacityError};
    /// # fn foo() -> Result<(), CapacityError> {
    /// let mut a = array_vec![5;];
    /// assert_eq!(a, []);
    ///
    /// a.try_resize_with(3, Default::default)?;
    /// assert_eq!(a, [0, 0, 0]);
    ///
    /// let mut g = 2;
    /// a.try_resize_with(5, move || { g += 1; g })?;
    /// assert_eq!(a, [0, 0, 0, 3, 4]);
    ///
    /// a.try_resize_with(1, || 1)?;
    /// assert_eq!(a, [0]);
    ///
    /// assert!(matches!(a.try_resize_with(10, || 7), Err(CapacityError)));
    /// # Ok(())
    /// # }
    /// # foo();
    /// ```
    ///
    /// [`try_resize`]: ArrayVec::try_resize
    /// [`resize_with`]: ArrayVec::resize_with
    /// [`Clone`]: core::clone::Clone
    /// [`Default`]: core::default::Default
    /// [`Default::default`]: core::default::Default::default
    #[inline]
    pub fn try_resize_with<F>(&mut self, new_len: usize, mut f: F) -> Result<(), CapacityError>
    where
        F: FnMut() -> T,
    {
        if new_len > Self::CAPACITY {
            return Err(CapacityError);
        }

        if new_len < self.len() {
            self.truncate(new_len);
            return Ok(());
        }

        while self.len() < new_len {
            unsafe { self.push_unchecked(f()) };
        }

        Ok(())
    }
}

impl<T, L, SM, const C: usize> ArrayVec<T, L, SM, C>
where
    T: Clone,
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    /// Resizes the array-vector in-place so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, the array-vector is extended by the difference,
    /// with each additional slot filled with `value`. If `new_len` is less than `len`,
    /// the array-vector is simply truncated.
    ///
    /// If you need only to resize to a smaller size, use [`truncate`].
    ///
    /// # Panics
    ///
    /// This method panics if `new_len > CAPACITY`. To avoid panic use [`try_resize`] which
    /// returns [`CapacityError`] instead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::array_vec;
    /// let mut a = array_vec![5;];
    /// assert_eq!(a, []);
    ///
    /// a.resize(2, 1);
    /// assert_eq!(a, [1, 1]);
    ///
    /// a.resize(4, 5);
    /// assert_eq!(a, [1, 1, 5, 5]);
    ///
    /// a.resize(1, 7);
    /// assert_eq!(a, [1]);
    /// ```
    ///
    /// [`truncate`]: ArrayVec::truncate
    /// [`try_resize`]: ArrayVec::try_resize
    #[inline]
    pub fn resize(&mut self, new_len: usize, value: T) {
        self.try_resize(new_len, value)
            .expect("insufficient capacity");
    }

    /// Tries to resize the array-vector in-place so that `len` is equal to `new_len`.
    ///
    /// This method returns [`CapacityError`] if `new_len > CAPACITY`.
    ///
    /// This is a non-panic version of [`resize`].
    ///
    /// If `new_len` is greater than `len`, the array-vector is extended by the difference,
    /// with each additional slot filled with `value`. If `new_len` is less than `len`,
    /// the array-vector is simply truncated.
    ///
    /// If you need only to resize to a smaller size, use [`truncate`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::{array_vec, errors::CapacityError};
    /// # fn foo() -> Result<(), CapacityError> {
    /// let mut a = array_vec![5; 1];
    /// assert_eq!(a, [1]);
    ///
    /// a.try_resize(5, 7)?;
    /// assert_eq!(a, [1, 7, 7, 7, 7]);
    ///
    /// a.try_resize(2, 0)?;
    /// assert_eq!(a, [1, 7]);
    ///
    /// assert!(matches!(a.try_resize(10, 7), Err(CapacityError)));
    /// # Ok(())
    /// # }
    /// # foo();
    /// ```
    ///
    /// [`truncate`]: ArrayVec::truncate
    /// [`resize`]: ArrayVec::resize
    pub fn try_resize(&mut self, new_len: usize, value: T) -> Result<(), CapacityError> {
        if new_len > Self::CAPACITY {
            return Err(CapacityError);
        }

        if new_len < self.len() {
            self.truncate(new_len);
            return Ok(());
        }

        while self.len() < new_len {
            unsafe { self.push_unchecked(value.clone()) };
        }

        Ok(())
    }

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

impl<T, L, SM, const C: usize> ArrayVec<T, L, SM, C>
where
    T: Copy,
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    /// Extends the array-vector by copying elements from a slice.
    ///
    /// This is optimized for [`Copy`] types, elements are copied bytewise.
    ///
    /// # Panics
    ///
    /// This method panics if there is no enough spare capacity to accommodate
    /// all elements from `s`. See [`try_copy_from_slice`] for a method that returns
    /// [`CapacityError`] instead.
    ///
    /// [`try_copy_from_slice`]: ArrayVec::try_copy_from_slice
    ///
    /// # Examples
    /// ```rust
    /// # use cds::array_vec;
    /// let mut a = array_vec![5; 1, 2];
    /// assert_eq!(a, [1, 2]);
    /// a.copy_from_slice(&[3, 4]);
    /// assert_eq!(a, [1, 2, 3, 4]);
    /// ```
    /// ```should_panic
    /// # use cds::array_vec;
    /// let mut a = array_vec![3; 1, 2];
    /// a.copy_from_slice(&[3, 4]);  // <-- this panics as there is only one spare slot
    /// ```
    #[inline]
    pub fn copy_from_slice(&mut self, s: &[T]) {
        if self.len() + s.len() > Self::CAPACITY {
            panic!("insufficient capacity");
        }
        unsafe { self.copy_from_slice_unchecked(s) };
    }

    /// Tries to extend the array-vector by copying elements from a slice.
    ///
    /// This is optimized for [`Copy`] types, elements are copied bytewise.
    ///
    /// This method is a non-panic version of [`copy_from_slice`].
    ///
    /// It returns [`CapacityError`] if there is no enough spare capacity to accommodate
    /// all elements from `s`.
    ///
    /// [`copy_from_slice`]: ArrayVec::copy_from_slice
    ///
    /// # Examples
    /// ```rust
    /// # use cds::{array_vec, errors::CapacityError};
    /// # fn foo() -> Result<(), CapacityError> {
    /// let mut a = array_vec![5; 1, 2];
    /// assert_eq!(a, [1, 2]);
    /// a.try_copy_from_slice(&[1, 2])?;
    /// assert_eq!(a, [1, 2, 1, 2]);
    /// assert!(matches!(a.try_copy_from_slice(&[3, 4]), Err(CapacityError)));
    /// # Ok(())
    /// # }
    /// # foo().unwrap();
    /// ```
    pub fn try_copy_from_slice(&mut self, s: &[T]) -> Result<(), CapacityError> {
        if self.len() + s.len() > Self::CAPACITY {
            return Err(CapacityError);
        }
        unsafe { self.copy_from_slice_unchecked(s) };
        Ok(())
    }

    /// Extends the array-vector by copying elements from a slice.
    ///
    /// This is optimized for [`Copy`] types, elements are copied bytewise.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is enough spare capacity to accommodate all elements
    /// from `s`.
    ///
    /// # Panics
    ///
    /// This method uses debug assertions to ensure that array-vector's capacity is not exceeded.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::array_vec;
    /// let mut a = array_vec![5; 1, 2];
    /// assert_eq!(a, [1, 2]);
    /// unsafe { a.copy_from_slice_unchecked(&[1, 2]) };
    /// assert_eq!(a, [1, 2, 1, 2]);
    /// ```
    #[inline]
    pub unsafe fn copy_from_slice_unchecked(&mut self, s: &[T]) {
        // SAFETY: it is impossible that regions overlap
        // as `self` cannot be borrowed as both mutable and immutable
        ptr::copy_nonoverlapping(s.as_ptr(), self.as_mut_ptr().add(self.len()), s.len());
        self.len += s.len();
    }
}

mod macros;
mod traits;

#[cfg(test)]
mod test_arrayvec;
