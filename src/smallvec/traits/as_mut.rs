use crate::{len::LengthType, mem::SpareMemoryPolicy, smallvec::SmallVec};
use core::convert::AsMut;

impl<T, L, SM, const C: usize> AsMut<[T]> for SmallVec<T, C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, L, SM, const C: usize> AsMut<SmallVec<T, C, L, SM>> for SmallVec<T, C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut SmallVec<T, C, L, SM> {
        self
    }
}

#[cfg(test)]
mod testing {
    use crate as cds;
    use crate::{small_vec, smallvec::SmallVec};
    use core::convert::AsMut;

    #[test]
    fn test_as_mut_slice() {
        let mut v = small_vec![3; u64; 1, 2, 3];
        let s: &mut [u64] = &mut [1, 2, 3];
        let v_s: &mut [u64] = v.as_mut();
        assert_eq!(v_s, s);
    }

    #[test]
    fn test_as_mut_av() {
        type SV = SmallVec<u64, 3>;
        let mut v = SV::try_from([1, 2, 3]).unwrap();
        let v_ref: &mut SV = v.as_mut();
        assert_eq!((v_ref as *const SV), (&v as *const SV));
    }
}
