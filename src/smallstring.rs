//! A string with "small size" optimization.

use crate::{
    len::{LengthType, Usize},
    mem::{
        alloc::{alloc_buffer, alloc_buffer_hae, realloc_buffer_hae, NOHAE},
        errors::ReservationError,
        SpareMemoryPolicy, Uninitialized,
    },
    smallstring::buffer::Buffer,
};
#[cfg(test)]
use core::mem::MaybeUninit;
use core::{ptr, slice};

mod buffer;

/// A UTF-8–encoded, growable string, with "small size" optimization.
///
/// Written as `SmallString<C, L, SM>`, small-string has local capacity of `C` bytes.
///
/// It uses type `L` as [`length type`], and `SM` as [`spare memory policy`].
///
/// Similar to [`str`] `SmallString` is UTF-8 encoded.
///
/// `SmallString` doesn't allocate memory, unless required capacity exceeds the local capacity.
/// In which case, small-string allocates a heap buffer, copies the data from local buffer to the
/// heap, and continues its operation from there.
///
/// [`spare memory policy`]: SpareMemoryPolicy
/// [`length type`]: LengthType
pub struct SmallString<const C: usize, L = Usize, SM = Uninitialized>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    /// The buffer
    buf: Buffer<C, L, SM>,

    /// The length of small string when local; the capacity of the buffer when on heap
    capacity: L,
}

impl<const C: usize, L, SM> SmallString<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    /// Creates a new empty small string.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::smallstring::SmallString;
    /// type S = SmallString<16>;
    /// let s = S::new();
    /// assert!(s.is_empty());
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            buf: Buffer::<C, L, SM>::new(),
            capacity: L::new(0),
        }
    }

    #[inline]
    unsafe fn heap_buffer(capacity: usize) -> Buffer<C, L, SM> {
        let p = alloc_buffer_hae(capacity);
        SM::init(p, capacity);
        Buffer::heap(p, 0)
    }

    /// Creates a new empty `SmallString` with at least the specified capacity.
    ///
    /// `SmallString`s have an internal local buffer of size `C` to hold their data. When the required
    /// capacity exceeds `C`, `SmallString`s allocate a heap buffer, copy the local buffer into it and
    /// continue the operation from there. This method creates an empty `SmallString`, but with the initial
    /// buffer that can hold at least `capacity` bytes. If `capacity <= C`, this method is equivalent to
    /// [`new`], i.e. creates an empty `SmallString` with local capacity of `C` bytes. Otherwise,
    /// a small string is created with a heap buffer of exactly `capacity` bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::smallstring::SmallString;
    /// let s = SmallString::<32>::with_capacity(16);
    /// assert_eq!(s.capacity(), 32);
    ///
    /// let s = SmallString::<32>::with_capacity(33);
    /// assert_eq!(s.capacity(), 33);
    /// ```
    ///
    /// [`new`]: Self::new
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity <= C {
            Self::new()
        } else {
            Self {
                buf: unsafe { Self::heap_buffer(capacity) },
                capacity: L::new(capacity),
            }
        }
    }

    /// Returns the string's capacity in bytes.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity.as_usize().max(C)
    }

    /// Returns the remaining spare capacity of the SmallString as a slice of `MaybeUninit<u8>`.
    #[cfg(test)]
    pub(crate) fn spare_capacity(&self) -> &[MaybeUninit<u8>] {
        let cap = self.capacity.as_usize();
        if cap <= C {
            unsafe {
                slice::from_raw_parts(
                    self.buf.local_ptr().add(cap) as *const MaybeUninit<u8>,
                    C - cap,
                )
            }
        } else {
            let (len, p) = self.buf.heap_len_p();
            let len = len.as_usize();
            unsafe { slice::from_raw_parts(p.add(len) as *const MaybeUninit<u8>, cap - len) }
        }
    }

    /// Returns the length of the array-string in bytes.
    ///
    /// Note that the returned length is in bytes, not chars or graphemes.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::small_str;
    /// let s = small_str![16; "€"];
    /// assert_eq!(s.len(), 3); // the length of array-string's UTF-8 encoding in bytes
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        let cap = self.capacity.as_usize();
        if cap <= C {
            cap
        } else {
            self.buf.heap_len().as_usize()
        }
    }

    /// Checks of the small-string is empty.
    ///
    /// Returns `true` if this `SmallString` has a length of zero, and `false` otherwise.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::small_str;
    /// let s = small_str![16;];
    /// assert!(s.is_empty());
    /// assert_eq!(s.len(), 0);
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a byte slice of this `SmallString`'s contents.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::small_str;
    /// let s = small_str![16; "cds"];
    /// assert_eq!(s.as_bytes(), &[99, 100, 115]);
    /// ```
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        let cap = self.capacity.as_usize();
        if cap <= C {
            unsafe { slice::from_raw_parts(self.buf.local_ptr(), cap) }
        } else {
            let (len, p) = self.buf.heap_len_p();
            unsafe { slice::from_raw_parts(p, len.as_usize()) }
        }
    }

    #[inline]
    fn as_bytes_mut(&mut self) -> &mut [u8] {
        let cap = self.capacity.as_usize();
        if cap <= C {
            unsafe { slice::from_raw_parts_mut(self.buf.local_mut_ptr(), cap) }
        } else {
            let (len, p) = self.buf.heap_len_mut_p();
            unsafe { slice::from_raw_parts_mut(p, len.as_usize()) }
        }
    }

    /// Extracts a string slice containing the entire `SmallString`.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::small_str;
    /// let s = small_str![16; "cds"];
    /// assert_eq!(s.as_str(), "cds");
    /// ```
    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(self.as_bytes()) }
    }

    /// Extracts a mutable string slice containing the entire `SmallString`.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::small_str;
    /// let mut s = small_str![16; "cds"];
    /// assert_eq!(s, "cds");
    ///
    /// s.as_mut_str().make_ascii_uppercase();
    /// assert_eq!(s, "CDS");
    /// ```
    #[inline]
    pub fn as_mut_str(&mut self) -> &mut str {
        unsafe { core::str::from_utf8_unchecked_mut(self.as_bytes_mut()) }
    }

    /// Truncates this small-string, removing all contents.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::small_str;
    /// let mut s = small_str![16; "cds"];
    /// assert_eq!(s, "cds");
    /// s.clear();
    /// assert_eq!(s, "");
    /// assert!(s.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        let cap = self.capacity.as_usize();
        if cap <= C {
            let p = self.buf.local_mut_ptr();
            unsafe {
                SM::init(p, cap);
            }
            self.capacity = L::new(0);
        } else {
            let (l, p) = self.buf.heap_len_mut_p();
            unsafe {
                SM::init(p, l.as_usize());
            }
            self.buf.set_heap_len(L::new(0));
        }
    }

    /// Truncates the `SmallString` to a specified length in bytes.
    ///
    /// If `new_len` is equal or greater than current small-string length, this method does nothing.
    ///
    /// # Panics
    ///
    /// This method panics of `new_len` doesn't lie on character boundary.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::small_str;
    /// let mut s = small_str![8; "cds!"];
    /// assert_eq!(s, "cds!");
    /// s.truncate(2);
    /// assert_eq!(s, "cd");
    /// ```
    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        let len;
        let cap = self.capacity.as_usize();

        let (l, p) = if cap <= C {
            len = cap;
            (&mut self.capacity, self.buf.local_mut_ptr())
        } else {
            let (hl, hp) = self.buf.heap_mut_len_mut_p();
            len = hl.as_usize();
            (hl, hp)
        };

        if new_len >= len {
            return;
        }

        let slc = unsafe { core::str::from_utf8_unchecked(slice::from_raw_parts(p, len)) };
        if !slc.is_char_boundary(new_len) {
            panic!("new_len doesn't lie on character boundary");
        }

        if !SM::NOOP {
            unsafe {
                SM::init(p.add(new_len), len - new_len);
            }
        }

        *l = L::new(new_len);
    }

    /// Reserves capacity for at least `additional` bytes more that the current length.
    ///
    /// The allocator may reserve more space to speculatively avoid frequent allocations.
    /// After calling `reserve`, capacity will be greater than or equal to `self.len() + additional`.
    /// Does nothing if capacity is already sufficient.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.reserve_impl(additional);
    }

    /// Reserves the minimum capacity for at least `additional` bytes more than the current length.
    ///
    /// Unlike [`reserve`], this will not deliberately over-allocate to speculatively avoid frequent
    /// allocations. After calling `reserve_exact`, capacity will be greater than or equal to
    /// `self.len() + additional`. Does nothing if the capacity is already sufficient.
    ///
    /// [`reserve`]: Self::reserve
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.reserve_exact_impl(additional);
    }

    #[inline(never)]
    fn reserve_impl(&mut self, additional: usize) -> (&mut L, *mut u8) {
        self.reserve_core(additional, |l, a| {
            l.checked_add_usize(a)
                .expect("SmallString capacity overflow")
                .next_power_of_two_or_max()
        })
    }

    #[inline(never)]
    fn reserve_exact_impl(&mut self, additional: usize) -> (&mut L, *mut u8) {
        self.reserve_core(additional, |l, a| {
            l.checked_add_usize(a)
                .expect("SmallString capacity overflow")
        })
    }

    #[inline]
    fn reserve_core<F>(&mut self, additional: usize, nc: F) -> (&mut L, *mut u8)
    where
        F: FnOnce(L, usize) -> L,
    {
        let cap = self.capacity.as_usize();

        if cap <= C {
            let len = cap;
            let cap = C;
            if additional <= cap - len {
                return (&mut self.capacity, self.buf.local_mut_ptr());
            }

            let new_cap = nc(self.capacity, additional);
            debug_assert!(new_cap > cap);

            let p = unsafe {
                let prefix = if SM::NOOP {
                    len
                } else {
                    cap /* copy spare memory too */
                };
                let tmp = alloc_buffer_hae::<u8>(new_cap.as_usize());
                ptr::copy_nonoverlapping(self.buf.local_ptr(), tmp, prefix);
                if !SM::NOOP {
                    SM::init(tmp.add(cap), new_cap.as_usize() - cap);
                    SM::init(self.buf.local_mut_ptr(), len);
                }
                tmp
            };
            self.buf.set_heap(p, self.capacity);
            self.capacity = new_cap;
            self.buf.heap_mut_len_mut_p()
        } else {
            let len = self.buf.heap_len();
            if additional <= cap - len.as_usize() {
                return self.buf.heap_mut_len_mut_p();
            }

            let new_cap = nc(len, additional);
            debug_assert!(new_cap > cap);

            let p = unsafe {
                let tmp = realloc_buffer_hae::<u8, SM>(
                    self.buf.heap_mut_ptr(),
                    len.as_usize(),
                    cap,
                    new_cap.as_usize(),
                );
                SM::init(tmp.add(cap), new_cap.as_usize() - cap);
                tmp
            };
            self.buf.set_heap_ptr(p);
            self.capacity = new_cap;
            self.buf.heap_mut_len_mut_p()
        }
    }

    unsafe fn try_from_bytes(bytes: &[u8]) -> Result<Self, ReservationError> {
        let len = bytes.len();
        if len > L::MAX {
            return Err(ReservationError::CapacityOverflow);
        }
        if len <= C {
            Ok(Self {
                buf: Buffer::<C, L, SM>::local_from_bytes(bytes),
                capacity: L::new(len),
            })
        } else {
            let p = alloc_buffer::<u8, NOHAE>(len)?;
            ptr::copy_nonoverlapping(bytes.as_ptr(), p, len);
            Ok(Self {
                buf: Buffer::<C, L, SM>::heap(p, len),
                capacity: L::new(len),
            })
        }
    }

    /// Appends a given string slice onto the end of this `SmallString`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::small_str;
    /// let mut s = small_str![3; "abc"];
    /// assert_eq!(s, "abc");
    /// s.push_str("def");
    /// assert_eq!(s, "abcdef");
    /// ```
    ///
    /// # Panics
    ///
    /// This method panics if new capacity exceeds `min(L::MAX, isize::MAX)`.
    #[inline]
    pub fn push_str(&mut self, s: &str) {
        self.extend_from_slice(s)
    }

    #[inline]
    fn extend_from_slice(&mut self, s: &str) {
        let len;
        let cap = self.capacity.as_usize();
        let s_len = s.len();

        let (l, p) = if cap <= C {
            len = cap;
            let cap = C;
            if cap - len >= s_len {
                (&mut self.capacity, self.buf.local_mut_ptr())
            } else {
                self.reserve_impl(s_len)
            }
        } else {
            len = self.buf.heap_len().as_usize();
            if cap - len >= s_len {
                self.buf.heap_mut_len_mut_p()
            } else {
                self.reserve_impl(s_len)
            }
        };
        unsafe { ptr::copy_nonoverlapping(s.as_ptr(), p.add(len), s_len) };
        l.add_assign(s_len);
    }

    /// Appends `ch` to the end of this `SmallString`.
    ///
    /// # Examples
    /// ```
    /// # use cds::small_str;
    /// let mut s = small_str![32; "Hello, world"];
    /// assert_eq!(s, "Hello, world");
    ///
    /// s.push('!');
    /// assert_eq!(s, "Hello, world!");
    /// ```
    #[inline]
    pub fn push(&mut self, ch: char) {
        let len;
        let cap = self.capacity.as_usize();
        let u8len = ch.len_utf8();

        let (l, p) = if cap <= C {
            len = cap;
            if C - len >= u8len {
                (&mut self.capacity, self.buf.local_mut_ptr())
            } else {
                self.reserve_impl(u8len)
            }
        } else {
            len = self.buf.heap_len().as_usize();
            if cap - len >= u8len {
                self.buf.heap_mut_len_mut_p()
            } else {
                self.reserve_impl(u8len)
            }
        };

        let dst = unsafe { slice::from_raw_parts_mut(p.add(len), u8len) };
        l.add_assign(ch.encode_utf8(dst).len());
    }

    /// Removes the last character from this `SmallString` and returns it.
    ///
    /// Returns `None` if this small-string is empty.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::small_str;
    /// let mut s = small_str![8; "cds"];
    /// assert_eq!(s.pop(), Some('s'));
    /// assert_eq!(s.pop(), Some('d'));
    /// assert_eq!(s.pop(), Some('c'));
    /// assert_eq!(s.pop(), None);
    /// ```
    #[inline]
    pub fn pop(&mut self) -> Option<char> {
        let len;
        let cap = self.capacity.as_usize();

        let (l, p) = if cap <= C {
            len = cap;
            (&mut self.capacity, self.buf.local_mut_ptr())
        } else {
            let (hl, hp) = self.buf.heap_mut_len_mut_p();
            len = hl.as_usize();
            (hl, hp)
        };

        let s = unsafe {
            let slc = slice::from_raw_parts_mut(p, len);
            core::str::from_utf8_unchecked(slc)
        };

        let ch = s.chars().next_back()?;
        let ch_len = ch.len_utf8();

        let new_len = len - ch_len;
        unsafe {
            SM::init(p.add(new_len), ch_len);
        }
        *l = L::new(new_len);
        Some(ch)
    }

    /// Inserts a character into this `SmallString` at a byte position.
    ///
    /// This is an `O(n)` operation as it may require copying every element in the buffer.
    ///
    /// # Panics
    ///
    /// This method panics if `idx` is larger than small-string's length, or doesn't lie on
    /// character boundary.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::small_str;
    /// let mut s = small_str![4; "ac"];
    /// s.insert(1, 'b');
    /// assert_eq!(s, "abc");
    /// ```
    #[inline]
    pub fn insert(&mut self, idx: usize, ch: char) {
        let len;
        let cap = self.capacity.as_usize();
        let ch_len = ch.len_utf8();

        let (l, p) = if cap <= C {
            len = cap;
            if C - len >= ch_len {
                (&mut self.capacity, self.buf.local_mut_ptr())
            } else {
                self.reserve_impl(ch_len)
            }
        } else {
            let (hl, hp) = self.buf.heap_mut_len_mut_p();
            len = hl.as_usize();
            if cap - len >= ch_len {
                (hl, hp)
            } else {
                self.reserve_impl(ch_len)
            }
        };

        let slc = unsafe { core::str::from_utf8_unchecked(slice::from_raw_parts(p, len)) };
        if !slc.is_char_boundary(idx) {
            panic!("idx doesn't lie on character boundary");
        }
        unsafe {
            let dst = p.add(idx);
            ptr::copy(dst, dst.add(ch_len), len - idx);
            ch.encode_utf8(slice::from_raw_parts_mut(dst, ch_len));
        }
        l.add_assign(ch_len);
    }

    /// Inserts a string slice into this `SmallString` at a byte position.
    ///
    /// This is an O(n) operation, as it potentially copies all bytes in the small-string.
    ///
    /// # Panics
    ///
    /// This method panics if `idx` is greater than small-string's length, or doesn't lie on
    /// character boundary.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cds::small_str;
    /// let mut s = small_str![8; ", world!"];
    /// s.insert_str(0, "Hello");
    /// assert_eq!(s, "Hello, world!");
    /// ```
    #[inline]
    pub fn insert_str(&mut self, idx: usize, s: &str) {
        let len;
        let cap = self.capacity.as_usize();
        let s_len = s.len();

        let (l, p) = if cap <= C {
            len = cap;
            if C - len >= s_len {
                (&mut self.capacity, self.buf.local_mut_ptr())
            } else {
                self.reserve_impl(s_len)
            }
        } else {
            let (hl, hp) = self.buf.heap_mut_len_mut_p();
            len = hl.as_usize();
            if cap - len >= s_len {
                (hl, hp)
            } else {
                self.reserve_impl(s_len)
            }
        };

        let slc = unsafe { core::str::from_utf8_unchecked(slice::from_raw_parts(p, len)) };
        if !slc.is_char_boundary(idx) {
            panic!("idx doesn't lie on character boundary");
        }
        unsafe {
            let dst = p.add(idx);
            ptr::copy(dst, dst.add(s_len), len - idx);
            ptr::copy_nonoverlapping(s.as_ptr(), dst, s_len);
        }
        l.add_assign(s_len);
    }

    /// Removes a [char] from `SmallString` at a byte position and returns it.
    ///
    /// This is a O(n) operation, as it potentially copies all elements in the buffer.
    ///
    /// # Panics
    ///
    /// This method panics of `idx` is larger than or equal to the `SmallString`'s length, or
    /// doesn't lie on character boundary.
    ///
    /// # Examples
    /// ```rust
    /// # use cds::small_str;
    /// let mut s = small_str![8; "cds"];
    /// assert_eq!(s.remove(1), 'd');
    /// assert_eq!(s, "cs");
    /// ```
    #[inline]
    pub fn remove(&mut self, idx: usize) -> char {
        let len;
        let cap = self.capacity.as_usize();

        let (l, p) = if cap <= C {
            len = cap;
            (&mut self.capacity, self.buf.local_mut_ptr())
        } else {
            let (hl, hp) = self.buf.heap_mut_len_mut_p();
            len = hl.as_usize();
            (hl, hp)
        };

        let slc = unsafe { core::str::from_utf8_unchecked(slice::from_raw_parts(p, len)) };
        let removed = match slc[idx..].chars().next() {
            Some(c) => c,
            None => panic!("idx is greater than or equal to the current length"),
        };

        let removed_len = removed.len_utf8();
        let new_len = len - removed_len;

        unsafe {
            let dst = p.add(idx);
            ptr::copy(dst.add(removed_len), dst, new_len - idx);
            if !SM::NOOP {
                SM::init(p.add(new_len), removed_len);
            }
        }

        *l = L::new(new_len);

        removed
    }
}

mod macros;
mod traits;

#[cfg(test)]
mod test_smallstring;
