use crate::{
    len::LengthType,
    mem::SpareMemoryPolicy,
    smallvec::{clone_from_slice_unchecked, SmallVec, DOHAE},
};
use core::{clone::Clone, mem};

impl<T, L, SM, const C: usize> Clone for SmallVec<T, C, L, SM>
where
    T: Clone,
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn clone(&self) -> Self {
        let src = self.as_slice();
        let src_len = src.len();
        let mut tmp = Self::new();
        let (len, p) = if mem::size_of::<T>() == 0 || src_len <= C {
            (&mut tmp.capacity, tmp.buf.local_mut_ptr())
        } else {
            tmp.try_reserve_exact_impl::<DOHAE>(src_len)
                .expect("smallvec clone failed to reserve")
        };
        unsafe {
            clone_from_slice_unchecked(src, len, p);
        }
        tmp
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.clear();
        let src = source.as_slice();
        let src_len = src.len();

        let cap = self.capacity.as_usize();
        let (len, p) = if mem::size_of::<T>() == 0 || cap <= C {
            if mem::size_of::<T>() == 0 || src_len <= C {
                (&mut self.capacity, self.buf.local_mut_ptr())
            } else {
                self.try_reserve_exact_impl::<DOHAE>(src_len)
                    .expect("smallvec clone_from failed to reserve")
            }
        } else if src_len <= cap {
            self.buf.heap_mut_len_mut_p()
        } else {
            self.try_reserve_exact_impl::<DOHAE>(src_len)
                .expect("smallvec clone_from failed to reserve")
        };

        unsafe {
            clone_from_slice_unchecked(src, len, p);
        }
    }
}
