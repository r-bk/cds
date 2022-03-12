use crate::{len::LengthType, mem::SpareMemoryPolicy, smallvec::SmallVec};
use core::convert::AsRef;

impl<T, L, SM, const C: usize> AsRef<[T]> for SmallVec<T, C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, L, SM, const C: usize> AsRef<SmallVec<T, C, L, SM>> for SmallVec<T, C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

#[cfg(test)]
mod testing {
    use crate as cds;
    use crate::{small_vec, smallvec::SmallVec};
    use core::convert::AsRef;

    #[test]
    fn test_as_ref_slice() {
        let v = small_vec![3; u64; 1, 2, 3];
        let s: &[u64] = &[1, 2, 3];
        let v_s: &[u64] = v.as_ref();
        assert_eq!(v_s, s);
    }

    #[test]
    fn test_as_ref_av() {
        type SV = SmallVec<u64, 3>;
        let v = SV::try_from([1, 2, 3]).unwrap();
        let v_ref: &SV = v.as_ref();
        assert_eq!((v_ref as *const SV), (&v as *const SV));
    }
}
