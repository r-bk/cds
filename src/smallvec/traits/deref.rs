use crate::{len::LengthType, mem::SpareMemoryPolicy, smallvec::SmallVec};
use core::ops::{Deref, DerefMut};

impl<T, L, SM, const C: usize> Deref for SmallVec<T, C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, L, SM, const C: usize> DerefMut for SmallVec<T, C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

#[cfg(test)]
mod testing {
    use crate as cds;
    use cds::small_vec;

    #[test]
    fn test_deref() {
        let mut v = small_vec![4; u64; 1, 2, 3];
        assert!(v.is_local());
        assert_eq!(*v, [1, 2, 3]);

        v.reserve(10);
        assert!(v.is_heap());
        assert_eq!(*v, [1, 2, 3]);
    }

    #[test]
    fn test_deref_mut() {
        let mut v = small_vec![4; u64; 1, 2, 3];
        assert!(v.is_local());
        assert_eq!(&mut *v, [1, 2, 3]);

        v.reserve(10);
        assert!(v.is_heap());
        assert_eq!(&mut *v, [1, 2, 3]);
    }
}
