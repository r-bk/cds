use crate::{
    len::LengthType,
    mem::{alloc::dealloc_buffer, SpareMemoryPolicy},
    smallstring::SmallString,
};

impl<const C: usize, L, SM> Drop for SmallString<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    fn drop(&mut self) {
        self.clear();

        let cap = self.capacity.as_usize();
        if cap > C {
            dealloc_buffer(self.buf.heap_mut_ptr(), cap);
        }
    }
}
