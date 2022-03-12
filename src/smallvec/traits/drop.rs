use crate::{len::LengthType, mem::SpareMemoryPolicy, smallvec::SmallVec};
use ::alloc::alloc::{self, Layout};
use core::mem;

impl<T, const C: usize, L, SM> Drop for SmallVec<T, C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    fn drop(&mut self) {
        self.truncate(0);

        let cap = self.capacity.as_usize();
        if mem::size_of::<T>() != 0 && cap > C {
            // SAFETY: cap > C means reserve_impl has succeeded at least once.
            // Hence, array_size cannot overflow because reserve_impl uses the safe function
            // to calculate new_layout.
            unsafe {
                let array_size = mem::size_of::<T>() * cap;
                let layout = Layout::from_size_align_unchecked(array_size, mem::align_of::<T>());
                alloc::dealloc(self.buf.heap_mut_ptr().cast(), layout);
            }
        }
    }
}

#[cfg(test)]
mod testing {
    use crate as cds;
    use cds::{
        len::U8,
        mem::Pattern,
        smallvec::SmallVec,
        testing::dropped::{Dropped, Track},
    };

    unsafe fn check_spare_mem(mut s: *const u8, e: *const u8, p: u8) {
        while s < e {
            assert_eq!(s.read(), p);
            s = s.add(1);
        }
    }

    #[test]
    fn test_drop_truncate_local() {
        const PATTERN: u8 = 0xDC;
        type SV<'a> = SmallVec<Dropped<'a, 16>, 16, U8, Pattern<PATTERN>>;
        let t = Track::<16>::new();

        let mut v = SV::from_iter(t.take(5));
        assert_eq!(t.n_allocated(), 5);
        assert!(t.dropped_range(0..0)); // empty range
        assert!(v.is_local());

        let p = v.as_ptr();
        v.truncate(0);
        unsafe { check_spare_mem(p.cast(), p.add(v.capacity()).cast(), PATTERN) };

        drop(v);
        assert!(t.dropped_range(0..5));
    }

    #[test]
    fn test_drop_truncate_heap() {
        const PATTERN: u8 = 0xED;
        type SV<'a> = SmallVec<Dropped<'a, 16>, 1, U8, Pattern<PATTERN>>;
        let t = Track::<16>::new();

        let mut v = SV::from_iter(t.take(5));
        assert_eq!(t.n_allocated(), 5);
        assert!(t.dropped_range(0..0)); // empty range
        assert!(v.is_heap());

        let p = v.as_ptr();
        v.truncate(0);
        unsafe { check_spare_mem(p.cast(), p.add(v.capacity()).cast(), PATTERN) };

        drop(v);
        assert!(t.dropped_range(0..5));
    }
}
