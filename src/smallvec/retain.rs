use crate::{len::LengthType, mem::SpareMemoryPolicy, smallvec::SmallVec};
use core::ptr;

pub struct RetainGuard<'a, T, const C: usize, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
    T: 'a,
{
    pub(super) sv: &'a mut SmallVec<T, C, L, SM>,
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

        // move the unprocessed elements to preserve small-vector's contiguity.
        // Set the correct length, and invoke spare memory policy if needed.
        unsafe {
            if self.deleted > 0 {
                ptr::copy(
                    self.sv.as_ptr().add(self.processed),
                    self.sv.as_mut_ptr().add(self.processed - self.deleted),
                    self.len - self.processed,
                );

                SM::init(self.sv.as_mut_ptr().add(new_len), self.deleted)
            }
            self.sv.set_len(new_len);
        }
    }
}
