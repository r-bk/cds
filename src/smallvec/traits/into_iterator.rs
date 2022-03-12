use crate::{len::LengthType, mem::SpareMemoryPolicy, smallvec::SmallVec};
use core::{iter::IntoIterator, slice};

impl<'a, T, L, SM, const C: usize> IntoIterator for &'a SmallVec<T, C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, L, SM, const C: usize> IntoIterator for &'a mut SmallVec<T, C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[cfg(test)]
mod testing {
    use crate as cds;
    use cds::smallvec::SmallVec;
    type SV = SmallVec<u64, 7>;

    fn use_iterator(v: &SV) -> u64 {
        let mut sum = 0;
        for e in v {
            sum += *e;
        }
        sum
    }

    fn use_iterator_mut(v: &mut SV) -> u64 {
        let mut sum = 0;
        for e in v {
            sum += *e;
        }
        sum
    }

    #[test]
    fn test_into_iterator_ref() {
        let v = SV::from_iter(0..3);
        use_iterator(&v);
    }

    #[test]
    fn test_into_iterator_mut() {
        let mut v = SV::from_iter(0..3);
        use_iterator_mut(&mut v);
    }
}
