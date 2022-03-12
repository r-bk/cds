//! A string-like array.

use crate::{
    errors::{CapacityError, IndexError, InsertError},
    len::{LengthType, Usize},
    mem::{SpareMemoryPolicy, Uninitialized},
};
use core::{marker::PhantomData, mem, ptr, slice};

/// A non-growable array with string-like API.
///
/// Written as `ArrayString<C, L, SM>`, array-string has the capacity of `C` bytes.
///
/// It uses type `L` as [`length type`], and `SM` as [`spare memory policy`].
///
/// Similar to [`str`] `ArrayString` is UTF-8 encoded.
///
/// [`spare memory policy`]: SpareMemoryPolicy
/// [`length type`]: LengthType
pub struct ArrayString<const C: usize, L = Usize, SM = Uninitialized>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    arr: [mem::MaybeUninit<u8>; C],
    len: L,
    phantom: PhantomData<SM>,
}

impl<L, SM, const C: usize> ArrayString<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    /// The capacity of the array-string as associated constant.
    ///
    /// The capacity can also be obtained via the [`capacity`] method.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::{arraystring::ArrayString, len::U8};
    /// type S = ArrayString<8, U8>;
    /// let s = S::new();
    /// assert_eq!(S::CAPACITY, 8);
    /// assert_eq!(s.capacity(), S::CAPACITY);
    /// ```
    ///
    /// [`capacity`]: ArrayString::capacity
    pub const CAPACITY: usize = C;

    /// Returns the capacity of the array-string in bytes.
    ///
    /// This is a convenience method. The capacity of the array-string is known at compilation time
    /// and can be also obtained via the [`CAPACITY`] associated constant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::array_str;
    /// let mut s = array_str![17;];
    /// assert_eq!(s.capacity(), 17);
    /// ```
    ///
    /// [`CAPACITY`]: ArrayString::CAPACITY
    #[inline]
    pub fn capacity(&self) -> usize {
        Self::CAPACITY
    }

    /// Returns the length of unused capacity in bytes.
    ///
    /// Equivalent to `capacity() - len()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::array_str;
    /// let mut s = array_str![2;];
    /// assert_eq!(s.capacity(), 2);
    /// assert_eq!(s.spare_capacity(), 2);
    ///
    /// s.push('a');
    /// assert_eq!(s.capacity(), 2);
    /// assert_eq!(s.spare_capacity(), 1);
    /// ```
    #[inline]
    pub fn spare_capacity(&self) -> usize {
        Self::CAPACITY - self.len.as_usize()
    }

    /// Creates a new empty `ArrayString`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::{arraystring::ArrayString, len::U8};
    /// type AS = ArrayString<7, U8>;
    /// let s = AS::new();
    /// ```
    #[inline]
    pub fn new() -> Self {
        let mut s = Self::new_raw(0);
        unsafe { SM::init(s.as_mut_ptr(), Self::CAPACITY) };
        s
    }

    #[inline(always)]
    fn new_raw(len: usize) -> Self {
        Self {
            // it is safe to call `assume_init` to create an array of `MaybeUninit`
            arr: unsafe { mem::MaybeUninit::uninit().assume_init() },
            len: L::new(len),
            phantom: PhantomData,
        }
    }

    #[inline]
    fn as_ptr(&self) -> *const u8 {
        self.arr.as_ptr() as *const u8
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.arr.as_mut_ptr() as *mut u8
    }

    #[inline]
    unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= Self::CAPACITY);
        self.len.set(new_len);
    }

    #[inline]
    unsafe fn spare_capacity_mut(&mut self) -> &mut [u8] {
        slice::from_raw_parts_mut(self.as_mut_ptr().add(self.len()), self.spare_capacity())
    }

    /// Returns the length of the array-string in bytes.
    ///
    /// Note that the returned length is in bytes, not chars or graphemes.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::array_str;
    /// let s = array_str![16; "€"];
    /// assert_eq!(s.len(), 3); // the length of array-string's UTF-8 encoding in bytes
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.len.as_usize()
    }

    /// Checks of the `ArrayString` is empty.
    ///
    /// Returns `true` if this `ArrayString` has a length of zero, and `false` otherwise.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::array_str;
    /// let a = array_str![16;];
    /// assert!(a.is_empty());
    /// assert_eq!(a.len(), 0);
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a byte slice of this `ArrayString`'s contents.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::array_str;
    /// let s = array_str![16; "cds"];
    /// assert_eq!(s.as_bytes(), &[99, 100, 115]);
    /// ```
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len()) }
    }

    #[inline]
    fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len()) }
    }

    /// Extracts a string slice containing the entire `ArrayString`.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::array_str;
    /// let s = array_str![16; "cds"];
    /// assert_eq!(s.as_str(), "cds");
    /// ```
    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(self.as_bytes()) }
    }

    /// Converts an `ArrayString` into a mutable string slice.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::array_str;
    /// let mut s = array_str![16; "cds"];
    /// assert_eq!(s, "cds");
    ///
    /// s.as_mut_str().make_ascii_uppercase();
    /// assert_eq!(s, "CDS");
    /// ```
    #[inline]
    pub fn as_mut_str(&mut self) -> &mut str {
        unsafe { core::str::from_utf8_unchecked_mut(self.as_bytes_mut()) }
    }

    /// Truncates this `ArrayString`, removing all contents.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::array_str;
    /// let mut s = array_str![16; "cds"];
    /// assert_eq!(s, "cds");
    /// s.clear();
    /// assert_eq!(s, "");
    /// assert!(s.is_empty());
    /// ```
    pub fn clear(&mut self) {
        let len = self.len();
        unsafe {
            self.set_len(0);
            SM::init(self.as_mut_ptr(), len);
        }
    }

    /// Appends a character to the end of this `ArrayString`.
    ///
    /// # Panics
    ///
    /// This method panics if the array-string doesn't have enough spare capacity to
    /// accommodate the UTF-8 encoded character.
    ///
    /// See [`try_push`] for a method which returns [`CapacityError`] instead.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::array_str;
    /// let mut s = array_str![8;];
    /// assert_eq!(s, "");
    /// s.push('A');
    /// assert_eq!(s, "A");
    /// ```
    ///
    /// Panics if there is no spare capacity:
    /// ```should_panic
    /// # use cds::array_str;
    /// let mut s = array_str![2; "ab"];
    /// s.push('c');
    /// ```
    ///
    /// [`try_push`]: ArrayString::try_push
    #[inline]
    pub fn push(&mut self, ch: char) {
        if ch.len_utf8() > self.spare_capacity() {
            panic!("insufficient capacity");
        }
        unsafe { self.push_unchecked(ch) };
    }

    /// Tries to append a character to the end of this `ArrayString`.
    ///
    /// This is a non-panic version of [`push`].
    ///
    /// Returns [`CapacityError`] if there is no spare capacity to accommodate the UTF-8 encoded
    /// character.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::{array_str, errors::CapacityError};
    /// let mut s = array_str![3; "ab"];
    /// assert!(s.try_push('c').is_ok());
    /// assert!(matches!(s.try_push('d'), Err(CapacityError)));
    /// ```
    ///
    /// [`push`]: ArrayString::push
    #[inline]
    pub fn try_push(&mut self, ch: char) -> Result<(), CapacityError> {
        if ch.len_utf8() > self.spare_capacity() {
            return Err(CapacityError);
        }
        unsafe { self.push_unchecked(ch) };
        Ok(())
    }

    /// Appends a character to the end of this `ArrayString` without spare capacity check.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the array-string has enough spare capacity to accommodate the
    /// UTF-8 encoded character.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::array_str;
    /// const c: char = 'c';
    /// let mut s = array_str![3; "ab"];
    /// if s.spare_capacity() >= c.len_utf8() {
    ///     unsafe { s.push_unchecked(c) };
    /// }
    /// ```
    #[inline]
    pub unsafe fn push_unchecked(&mut self, ch: char) {
        let len = ch.encode_utf8(self.spare_capacity_mut()).len();
        self.len += len;
    }

    /// Appends a given string slice to the end of this `ArrayString`.
    ///
    /// # Panics
    ///
    /// The method panics if there is no enough spare capacity to accommodate the whole string
    /// slice.
    ///
    /// See [`try_push_str`] for a method which returns [`CapacityError`] instead.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::array_str;
    /// let mut s = array_str![16;];
    /// s.push_str("Hello, world!");
    /// assert_eq!("Hello, world!", s);
    /// ```
    ///
    /// Panics if there is no spare capacity:
    /// ```should_panic
    /// # use cds::array_str;
    /// let mut s = array_str![8; "Hello"];
    /// s.push_str(", world!");
    /// ```
    ///
    /// [`try_push_str`]: ArrayString::try_push_str
    #[inline]
    pub fn push_str(&mut self, s: &str) {
        if s.len() > self.spare_capacity() {
            panic!("insufficient capacity");
        }
        unsafe { self.push_str_unchecked(s) };
    }

    /// Appends as much characters of a string slice as spare capacity allows.
    ///
    /// Returns the number of bytes copied. Note that the return value represents the size
    /// of the UTF-8 encoding of the successfully copied characters.
    ///
    /// The difference between [`push_str`] and [`add_str`] is that the former panics if there
    /// is no spare capacity to accommodate the whole string slice, while the latter copies as much
    /// characters as possible.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::array_str;
    /// let mut s = array_str![4;];
    /// assert_eq!(s.add_str("€€"), 3);
    /// assert_eq!(s.add_str("€"), 0);
    /// assert_eq!(s, "€");
    /// ```
    ///
    /// [`push_str`]: ArrayString::push_str
    /// [`add_str`]: ArrayString::add_str
    #[inline]
    pub fn add_str(&mut self, s: &str) -> usize {
        let spare_bytes = self.spare_capacity();
        let mut s_len = s.len();
        if s_len > spare_bytes {
            s_len = spare_bytes;
            while !s.is_char_boundary(s_len) {
                s_len -= 1;
            }
        }
        unsafe {
            let len = self.len();
            ptr::copy_nonoverlapping(s.as_ptr(), self.as_mut_ptr().add(len), s_len);
            self.set_len(len + s_len);
        }
        s_len
    }

    /// Tries to append a given string slice to the end of this `ArrayString`.
    ///
    /// This is a non-panic version of [`push_str`].
    ///
    /// Returns [`CapacityError`] if there is no enough spare capacity to accommodate the whole
    /// string slice.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::{array_str, errors::CapacityError};
    /// let mut s = array_str![8;];
    /// assert!(s.try_push_str("Hello").is_ok());
    /// assert!(matches!(s.try_push_str(", world!"), Err(CapacityError)));
    /// ```
    ///
    /// [`push_str`]: ArrayString::push_str
    #[inline]
    pub fn try_push_str(&mut self, s: &str) -> Result<(), CapacityError> {
        if s.len() > self.spare_capacity() {
            return Err(CapacityError);
        }
        unsafe { self.push_str_unchecked(s) };
        Ok(())
    }

    /// Appends a given string slice to the end of this `ArrayString` without spare capacity check.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is enough spare capacity to push the whole string slice.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::array_str;
    /// const STR: &'static str = ", world!";
    /// let mut s = array_str![16; "Hello"];
    /// if STR.len() <= s.spare_capacity() {
    ///     unsafe { s.push_str_unchecked(STR) };
    /// }
    /// assert_eq!("Hello, world!", s);
    /// ```
    #[inline]
    pub unsafe fn push_str_unchecked(&mut self, s: &str) {
        let len = s.len();
        ptr::copy_nonoverlapping(s.as_ptr(), self.as_mut_ptr().add(self.len()), len);
        self.len += len;
    }

    /// Removes the last character from this `ArrayString` and returns it.
    ///
    /// Returns `None` if this array-string is empty.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::array_str;
    /// let mut s = array_str![8; "cds"];
    /// assert_eq!(s.pop(), Some('s'));
    /// assert_eq!(s.pop(), Some('d'));
    /// assert_eq!(s.pop(), Some('c'));
    /// assert_eq!(s.pop(), None);
    /// ```
    #[inline]
    pub fn pop(&mut self) -> Option<char> {
        let ch = self.chars().rev().next()?;
        let ch_len = ch.len_utf8();
        let new_len = self.len() - ch_len;
        unsafe {
            SM::init(self.as_mut_ptr().add(new_len), ch_len);
            self.set_len(new_len);
        }
        Some(ch)
    }

    /// Inserts a character into this `ArrayString` at a byte position.
    ///
    /// This is an O(n) operation, as it potentially copies all bytes in the array-string.
    ///
    /// # Panics
    ///
    /// This method panics if any of the following conditions is true:
    ///
    /// - `idx` doesn't lie on a [`char`] boundary
    /// - `idx` is greater than array-string's length
    /// - there is no spare capacity to accommodate the UTF-8 encoded character
    ///
    /// See [`try_insert`] for a method which returns [`InsertError`] instead.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::array_str;
    /// let mut s = array_str![8; "ac"];
    /// s.insert(1, 'b');
    /// assert_eq!("abc", s);
    /// ```
    ///
    /// [`try_insert`]: ArrayString::try_insert
    #[inline]
    pub fn insert(&mut self, idx: usize, ch: char) {
        self.try_insert(idx, ch).expect("insert failed")
    }

    /// Tries to insert a character into this `ArrayString` at a byte position.
    ///
    /// This is an O(n) operation, as it potentially copies all bytes in the array-string.
    ///
    /// This is a non-panic version of [`insert`].
    ///
    /// This method returns the following error:
    ///
    /// - [`InsertError::InvalidIndex`] - if `idx` doesn't lie on a [`char`] boundary,
    ///   or `idx > len`
    /// - [`InsertError::InsufficientCapacity`] - if there is no enough spare capacity to
    ///   accommodate the UTF-8 encoded character
    ///
    /// # Examples
    /// ```rust
    /// # use cds::{array_str, errors::InsertError};
    /// let mut s = array_str![6; "2"];
    /// assert!(s.try_insert(1, '€').is_ok());
    /// assert_eq!(s, "2€");
    /// assert!(matches!(s.try_insert(2, '5'), Err(InsertError::InvalidIndex))); // not a char boundary
    /// assert!(matches!(s.try_insert(5, '0'), Err(InsertError::InvalidIndex))); // index exceeds length
    /// assert!(matches!(s.try_insert(4, '€'), Err(InsertError::InsufficientCapacity)));
    /// ```
    ///
    /// [`insert`]: ArrayString::insert
    #[inline]
    pub fn try_insert(&mut self, idx: usize, ch: char) -> Result<(), InsertError> {
        if !self.is_char_boundary(idx) {
            return Err(InsertError::InvalidIndex);
        }

        let ch_len = ch.len_utf8();
        if ch_len > self.spare_capacity() {
            return Err(InsertError::InsufficientCapacity);
        }

        unsafe {
            let len = self.len();
            let tgt = self.as_mut_ptr().add(idx);
            ptr::copy(tgt, tgt.add(ch_len), len - idx);
            ch.encode_utf8(slice::from_raw_parts_mut(tgt, ch_len));
            self.set_len(len + ch_len);
        }

        Ok(())
    }

    /// Inserts a string slice into this `ArrayString` at a byte position.
    ///
    /// This is an O(n) operation, as it potentially copies all bytes in the array-string.
    ///
    /// # Panics
    ///
    /// This method panics if any of the following conditions is true:
    ///
    /// - `idx` doesn't lie on a [`char`] boundary
    /// - `idx` is greater then array-string length
    /// - there is no enough spare capacity to accommodate the whole string slice
    ///
    /// See [`try_insert_str`] for a method which returns [`InsertError`] instead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::array_str;
    /// let mut s = array_str![8; "ds"];
    /// s.insert_str(0, "c");
    /// assert_eq!(s, "cds");
    /// ```
    ///
    /// [`try_insert_str`]: ArrayString::try_insert_str
    #[inline]
    pub fn insert_str(&mut self, idx: usize, s: &str) {
        self.try_insert_str(idx, s).expect("insert_str failed")
    }

    /// Tries to insert a string slice into this `ArrayString` at a byte position.
    ///
    /// This is an O(n) operation, as it potentially copies all bytes in the array-string.
    ///
    /// This is a non-panic version of [`insert_str`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::{array_str, errors::InsertError};
    /// let mut s = array_str![5; "2"];
    /// assert!(s.try_insert_str(1, "€").is_ok());
    /// assert_eq!(s, "2€");
    /// assert!(matches!(s.try_insert_str(2, "a"), Err(InsertError::InvalidIndex)));
    /// assert!(matches!(s.try_insert_str(5, "a"), Err(InsertError::InvalidIndex)));
    /// assert!(matches!(s.try_insert_str(4, "€"), Err(InsertError::InsufficientCapacity)));
    /// ```
    ///
    /// [`insert_str`]: ArrayString::insert_str
    #[inline]
    pub fn try_insert_str(&mut self, idx: usize, s: &str) -> Result<(), InsertError> {
        if !self.is_char_boundary(idx) {
            return Err(InsertError::InvalidIndex);
        }

        let s_len = s.len();
        if s_len > self.spare_capacity() {
            return Err(InsertError::InsufficientCapacity);
        }

        unsafe {
            let len = self.len();
            let tgt = self.as_mut_ptr().add(idx);
            ptr::copy(tgt, tgt.add(s_len), len - idx);
            ptr::copy_nonoverlapping(s.as_ptr(), tgt, s_len);
            self.set_len(len + s_len);
        }

        Ok(())
    }

    /// Removes a [`char`] from the `ArrayString` at a byte position and returns it.
    ///
    /// This is an O(n) operation, as it potentially copies all bytes in the array-string.
    ///
    /// # Panics
    ///
    /// This method panics if any of the following conditions is true:
    ///
    /// - `idx` doesn't lie on a [`char`] boundary
    /// - `idx` is greater than or equal the array-string length
    ///
    /// See [`try_remove`] for a method which returns [`IndexError`] instead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::array_str;
    /// let mut s = array_str![8; "2€ "];
    /// assert_eq!(s.remove(1), '€');
    /// assert_eq!(s, "2 ");
    /// ```
    ///
    /// [`try_remove`]: ArrayString::try_remove
    #[inline]
    pub fn remove(&mut self, idx: usize) -> char {
        self.try_remove(idx).expect("invalid index")
    }

    /// Tries to remove a [`char`] from the `ArrayString` at a byte position and returns it.
    ///
    /// This is an O(n) operation, as it potentially copies all bytes in the array-string.
    ///
    /// This is a non-panic version of [`remove`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::{array_str, errors::IndexError};
    /// # fn foo() -> Result<(), IndexError> {
    /// let mut s = array_str![4; "2€"];
    /// for i in 2..=5 {
    ///     assert!(matches!(s.try_remove(i), Err(IndexError)));
    /// }
    /// assert_eq!(s.try_remove(0)?, '2');
    /// assert_eq!(s, "€");
    /// # Ok(())
    /// # }
    /// # foo().unwrap();
    /// ```
    ///
    /// [`remove`]: ArrayString::remove
    #[inline]
    pub fn try_remove(&mut self, idx: usize) -> Result<char, IndexError> {
        if !self.is_char_boundary(idx) {
            return Err(IndexError);
        }

        let ch = match self[idx..].chars().next() {
            Some(ch) => ch,
            None => return Err(IndexError),
        };

        let len = self.len();
        let ch_len = ch.len_utf8();
        let new_len = len - ch_len;
        let to_copy_len = new_len - idx;

        unsafe {
            let tgt = self.as_mut_ptr().add(idx);
            ptr::copy(tgt.add(ch_len), tgt, to_copy_len);
            SM::init(tgt.add(to_copy_len), ch_len);
            self.set_len(new_len);
        }

        Ok(ch)
    }

    /// Truncates the `ArrayString` to a specified length in bytes.
    ///
    /// If `new_len` is equal or greater than current array-string length, this method does nothing.
    ///
    /// # Panics
    ///
    /// This method panics if `new_len` doesn't lie on a [`char`] boundary.
    ///
    /// See [`try_truncate`] for a method which returns [`IndexError`] instead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::array_str;
    /// let mut s = array_str![4; "cds"];
    /// s.truncate(1);
    /// assert_eq!(s, "c");
    /// ```
    ///
    /// [`try_truncate`]: ArrayString::try_truncate
    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        self.try_truncate(new_len).expect("truncate failed")
    }

    /// Tries to truncate the `ArrayString` to a specified length in bytes.
    ///
    /// If `new_len` is equal or greater than current array-string length, this method does nothing.
    ///
    /// This is a non-panic version of [`truncate`].
    ///
    /// This method returns [`IndexError`] if `new_len` doesn't lie on a [`char`] boundary.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::{array_str, errors::IndexError};
    /// let mut s = array_str![8; "2€"];
    /// assert!(matches!(s.try_truncate(2), Err(IndexError))); // <-- 2 is not a char boundary
    /// assert!(s.try_truncate(4).is_ok());  // <-- new_len equals the current array-string length
    /// assert_eq!(s, "2€");
    /// assert!(s.try_truncate(1).is_ok());
    /// assert_eq!(s, "2");
    /// ```
    ///
    /// [`truncate`]: ArrayString::truncate
    #[inline]
    pub fn try_truncate(&mut self, new_len: usize) -> Result<(), IndexError> {
        let len = self.len();
        if new_len >= len {
            return Ok(());
        }

        if !self.is_char_boundary(new_len) {
            return Err(IndexError);
        }

        unsafe {
            self.set_len(new_len);
            SM::init(self.as_mut_ptr().add(new_len), len - new_len);
        }

        Ok(())
    }
}

mod format;
pub use format::*;

mod macros;
mod traits;

#[cfg(test)]
mod test_arraystring;
