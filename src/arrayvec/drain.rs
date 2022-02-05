use crate::{
    arrayvec::ArrayVec,
    defs::{LengthType, SpareMemoryPolicy},
};
use core::{
    fmt::{Debug, Formatter},
    iter::{DoubleEndedIterator, ExactSizeIterator, FusedIterator, Iterator},
    mem,
    ops::Drop,
    ptr, slice,
};

/// A draining iterator for [`ArrayVec`].
///
/// See [`ArrayVec::drain`] for more information.
#[allow(dead_code)]
pub struct Drain<'a, T, L, SM, const C: usize>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
    T: 'a,
{
    // the owner ArrayVec
    pub(super) av: ptr::NonNull<ArrayVec<T, L, SM, C>>,
    // an iterator over the slice to be drained
    pub(super) iter: slice::Iter<'a, T>,
    // the index of the first element past the drained range; or L::MAX for empty drained range
    pub(super) tail: L,
    // the length of the tail to preserve; or L::MAX for empty drained range
    pub(super) tail_len: L,
}

struct DropGuard<'s, 'a, T, L, SM, const C: usize>(&'s mut Drain<'a, T, L, SM, C>)
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
    T: 'a;

impl<'a, T, L, SM, const C: usize> Drain<'a, T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
    T: 'a,
{
    /// Returns the remaining items of this iterator as a slice.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self.iter.as_slice()
    }
}

impl<'a, T, L, SM, const C: usize> Debug for Drain<'a, T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
    T: 'a,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let s = self.iter.as_slice();
        write!(
            f,
            "arrayvec::Drain{{av: {:?}, iter: ({:?}, {}), tail: {}, tail_len: {}}}",
            self.av,
            s.as_ptr(),
            s.len(),
            self.tail,
            self.tail_len
        )
    }
}

impl<'a, T, L, SM, const C: usize> AsRef<[T]> for Drain<'a, T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
    T: 'a,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<'a, T, L, SM, const C: usize> Iterator for Drain<'a, T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
    T: 'a,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|rel| unsafe { ptr::read(rel as *const _) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T, L, SM, const C: usize> DoubleEndedIterator for Drain<'a, T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
    T: 'a,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next_back()
            .map(|rel| unsafe { ptr::read(rel as *const _) })
    }
}

impl<'a, T, L, SM, const C: usize> ExactSizeIterator for Drain<'a, T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
    T: 'a,
{
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, T, L, SM, const C: usize> FusedIterator for Drain<'a, T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
    T: 'a,
{
}

impl<'s, 'a, T, L, SM, const C: usize> Drop for DropGuard<'s, 'a, T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
    T: 'a,
{
    #[inline]
    fn drop(&mut self) {
        // Append the tail to the head to preserve array-vector's continuity,
        // and invoke SpareMemoryPolicy as needed.

        let av = unsafe { self.0.av.as_mut() };
        let head = av.len(); // `ArrayVec::drain` sets `len` to reflect the head only.
        let tail = self.0.tail.as_usize();
        let tail_len = self.0.tail_len.as_usize();

        debug_assert!(tail > head); // `ArrayVec::drain` must ensure this

        unsafe {
            if tail_len > 0 {
                let new_len = head + tail_len;
                if mem::size_of::<T>() != 0 {
                    let src = av.as_ptr().add(tail);
                    let dst = av.as_mut_ptr().add(head);
                    ptr::copy(src, dst, tail_len);
                    SM::init(av.as_mut_ptr().add(new_len), tail - head);
                }
                av.set_len(new_len);
            } else {
                SM::init(av.as_mut_ptr().add(head), tail - head);
            }
        }
    }
}

impl<'a, T, L, SM, const C: usize> Drop for Drain<'a, T, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
    T: 'a,
{
    #[inline]
    fn drop(&mut self) {
        if self.tail_len == L::MAX && self.tail == L::MAX {
            // empty drained range, nothing to do
            return;
        }

        let av = unsafe { self.av.as_mut() };

        if mem::size_of::<T>() == 0 {
            // ZST doesn't need any mem copy, just truncate the correct number of elements
            let head = av.len();
            let tail = self.tail.as_usize();
            let tail_len = self.tail_len.as_usize();
            unsafe { av.set_len(head + (tail - head) + tail_len) };
            av.truncate(head + tail_len);
        } else {
            // move the iterator to stack, to be able to borrow it read-only even when
            // `self` is borrowed for write in the DropGuard below
            let iter = mem::replace(&mut self.iter, (&[]).iter());

            // ensure array-vec continuity is preserved and SpareMemoryPolicy is invoked
            // even if one of the drained elements panics while dropped.
            let _guard = DropGuard(self);

            let remaining = iter.len();

            if remaining > 0 {
                // the iterator wasn't fully consumed, drop the remaining elements
                unsafe {
                    let ptr = iter.as_slice().as_ptr() as *mut T;
                    let s = slice::from_raw_parts_mut(ptr, remaining);
                    ptr::drop_in_place(s);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate as cds;
    use cds::array_vec;

    #[cfg(feature = "std")]
    #[test]
    fn test_drain_debug() {
        let mut a = array_vec![3; u64; 1];
        let ptr = a.as_ptr();
        let d = a.drain(0..1);
        let s = format!("{:?}", d);
        assert_eq!(
            s,
            format!(
                "arrayvec::Drain{{av: {:?}, iter: ({:?}, 1), tail: 1, tail_len: 0}}",
                ptr, ptr
            )
        );
    }

    #[test]
    fn test_drain_double_ended_iterator() {
        let mut a = array_vec![3; 1, 2, 3];
        assert_eq!(a, [1, 2, 3]);

        for (i, e) in a.drain(1..).rev().enumerate() {
            assert_eq!(e, 3 - i);
        }

        assert_eq!(a, [1]);
    }

    #[test]
    fn test_drain_as_slice() {
        let mut a = array_vec![3; 1, 2, 3];

        assert_eq!(a.drain(2..).as_slice(), [3]);
        assert_eq!(a.drain(..).as_slice(), [1, 2]);
    }

    #[test]
    fn test_drain_as_ref() {
        let mut a = array_vec![3; 1, 2, 3];

        assert_eq!(a.drain(2..).as_ref(), [3]);
        assert_eq!(a.drain(..).as_ref(), [1, 2]);
    }

    #[test]
    fn test_drain_size_hint() {
        let mut a = array_vec![3; 1, 2, 3];
        let mut d = a.drain(..);

        let mut i = 0;
        loop {
            if i > 3 {
                break;
            }

            d.next();
            let (min, max) = d.size_hint();
            assert_eq!(min, if i < 3 { 3 - i - 1 } else { 0 });
            assert_eq!(max, Some(min));

            i += 1;
        }
    }
}
