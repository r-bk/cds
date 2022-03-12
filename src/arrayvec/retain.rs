use crate::{arrayvec::ArrayVec, len::LengthType, mem::SpareMemoryPolicy};
use core::ptr;

pub struct RetainGuard<'a, T, const C: usize, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
    T: 'a,
{
    pub(super) av: &'a mut ArrayVec<T, C, L, SM>,
    pub(super) len: usize,
    pub(super) processed: usize,
    pub(super) deleted: usize,
}

impl<'a, T, L, SM, const C: usize> Drop for RetainGuard<'a, T, C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
    T: 'a,
{
    fn drop(&mut self) {
        debug_assert!(self.deleted <= self.processed);
        debug_assert!(self.processed <= self.len);

        let new_len = self.len - self.deleted;

        // move the unprocessed elements to preserve array-vector's contiguity.
        // Set the correct length, and invoke spare memory policy if needed.
        unsafe {
            if self.deleted > 0 {
                ptr::copy(
                    self.av.as_ptr().add(self.processed),
                    self.av.as_mut_ptr().add(self.processed - self.deleted),
                    self.len - self.processed,
                );

                SM::init(self.av.as_mut_ptr().add(new_len), self.deleted)
            }
            self.av.set_len(new_len);
        }
    }
}
