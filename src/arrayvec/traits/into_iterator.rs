use crate::{arrayvec::ArrayVec, len::LengthType, mem::SpareMemoryPolicy};
use core::{iter::IntoIterator, slice};

impl<'a, T, L, SM, const C: usize> IntoIterator for &'a ArrayVec<T, L, SM, C>
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

impl<'a, T, L, SM, const C: usize> IntoIterator for &'a mut ArrayVec<T, L, SM, C>
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
    use cds::{arrayvec::ArrayVec, len::Usize, mem::Uninitialized};
    type A = ArrayVec<u64, Usize, Uninitialized, 7>;

    fn use_iterator(a: &A) -> u64 {
        let mut sum = 0;
        for e in a {
            sum += *e;
        }
        sum
    }

    fn use_iterator_mut(a: &mut A) -> u64 {
        let mut sum = 0;
        for e in a {
            sum += *e;
        }
        sum
    }

    #[test]
    fn test_into_iterator_ref() {
        let a = A::from_iter(0..3);
        use_iterator(&a);
    }

    #[test]
    fn test_into_iterator_mut() {
        let mut a = A::from_iter(0..3);
        use_iterator_mut(&mut a);
    }
}
