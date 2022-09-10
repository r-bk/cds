use crate::{len::LengthType, mem::SpareMemoryPolicy, smallvec::SmallVec};
use core::{
    fmt::{Debug, Formatter},
    iter::{DoubleEndedIterator, ExactSizeIterator, FusedIterator, Iterator},
    mem,
    ops::Drop,
    ptr, slice,
};

/// A draining iterator for [`SmallVec`].
///
/// See [`SmallVec::drain`] for more information.
#[allow(dead_code)]
pub struct Drain<'a, T, L, SM, const C: usize>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
    T: 'a,
{
    // the owner SmallVec
    pub(super) sv: ptr::NonNull<SmallVec<T, C, L, SM>>,
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
            "smallvec::Drain{{sv: {:?}, iter: ({:?}, {}), tail: {}, tail_len: {}}}",
            self.sv,
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
        // Append the tail to the head to preserve small-vector's continuity,
        // and invoke SpareMemoryPolicy as needed.

        let sv = unsafe { self.0.sv.as_mut() };
        let (len, p) = sv.len_mut_p();
        let head = len.as_usize(); // `SmallVec::drain` sets `len` to reflect the head only.
        let tail = self.0.tail.as_usize();
        let tail_len = self.0.tail_len.as_usize();

        debug_assert!(tail > head); // `SmallVec::drain` must ensure this

        unsafe {
            if tail_len > 0 {
                let new_len = head + tail_len;
                if mem::size_of::<T>() != 0 {
                    let src = p.add(tail);
                    let dst = p.add(head);
                    ptr::copy(src, dst, tail_len);
                    SM::init(p.add(new_len), tail - head);
                }
                sv.set_len(new_len);
            } else {
                SM::init(p.add(head), tail - head);
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

        // move the iterator to stack, to be able to borrow it read-only even when
        // `self` is borrowed for write in the DropGuard below
        let iter = mem::replace(&mut self.iter, [].iter());
        let remaining = iter.len();

        let mut sv = self.sv;

        if mem::size_of::<T>() == 0 {
            let sv = unsafe { sv.as_mut() };
            // ZST doesn't need any mem copy, just truncate the correct number of elements
            let head = sv.len();
            let tail_len = self.tail_len.as_usize();
            unsafe { sv.set_len(head + remaining + tail_len) };
            sv.truncate(head + tail_len);
        } else {
            // ensure small-vec continuity is preserved and SpareMemoryPolicy is invoked
            // even if one of the drained elements panics while dropped.
            let _guard = DropGuard(self);

            if remaining > 0 {
                let p_to_drop = iter.as_slice().as_ptr();

                // the iterator wasn't fully consumed, drop the remaining elements
                unsafe {
                    let sv = sv.as_mut();
                    let p_sv = sv.as_mut_ptr();
                    let offset = p_to_drop.offset_from(p_sv);
                    let slc = ptr::slice_from_raw_parts_mut(p_sv.offset(offset), remaining);
                    ptr::drop_in_place(slc);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate as cds;
    use cds::small_vec;

    #[cfg(feature = "std")]
    #[test]
    fn test_drain_debug() {
        let mut v = small_vec![3; u64; 1];
        let ptr = v.as_ptr();
        let d = v.drain(0..1);
        let s = format!("{:?}", d);
        assert_eq!(
            s,
            format!(
                "smallvec::Drain{{sv: {:?}, iter: ({:?}, 1), tail: 1, tail_len: 0}}",
                ptr, ptr
            )
        );
    }

    #[test]
    fn test_drain_double_ended_iterator() {
        let mut v = small_vec![3; 1, 2, 3];
        assert_eq!(v, [1, 2, 3]);

        for (i, e) in v.drain(1..).rev().enumerate() {
            assert_eq!(e, 3 - i);
        }

        assert_eq!(v, [1]);
    }

    #[test]
    fn test_drain_as_slice() {
        let mut v = small_vec![3; 1, 2, 3];

        assert_eq!(v.drain(2..).as_slice(), [3]);
        assert_eq!(v.drain(..).as_slice(), [1, 2]);
    }

    #[test]
    fn test_drain_as_ref() {
        let mut v = small_vec![3; 1, 2, 3];

        assert_eq!(v.drain(2..).as_ref(), [3]);
        assert_eq!(v.drain(..).as_ref(), [1, 2]);
    }

    #[test]
    fn test_drain_size_hint() {
        let mut v = small_vec![3; 1, 2, 3];
        let mut d = v.drain(..);

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
