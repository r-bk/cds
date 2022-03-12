use crate::{arrayvec::ArrayVec, len::LengthType, mem::SpareMemoryPolicy};
use core::cmp::{Ord, Ordering, PartialOrd};

impl<T, L, SM, const C: usize> PartialOrd for ArrayVec<T, C, L, SM>
where
    T: PartialOrd,
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

impl<T, L, SM, const C: usize> Ord for ArrayVec<T, C, L, SM>
where
    T: Ord,
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

#[cfg(test)]
mod testing {
    use crate as cds;
    use cds::array_vec;
    use core::cmp::Ordering;

    #[test]
    fn test_partial_ord() {
        let a = array_vec![3; u64; 1, 2, 3];
        let b = array_vec![3; u64; 2, 2, 3];

        assert!(a < b);
        assert!(a <= b);
        assert!(!(a >= b));
        assert!(!(a > b));

        assert!(b > a);
        assert!(b >= a);
        assert!(!(b <= a));
        assert!(!(b < a));
    }

    #[test]
    fn test_ord() {
        let a = array_vec![3; u64; 1, 2, 3];
        let b = array_vec![3; u64; 2, 2, 3];

        assert_eq!(a.cmp(&b), Ordering::Less);
        assert_eq!(b.cmp(&a), Ordering::Greater);
        assert_eq!(a.cmp(&a), Ordering::Equal);
    }
}
