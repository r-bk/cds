use crate as cds;
use cds::{
    array_vec,
    arrayvec::{
        errors::{InsertError, InsertErrorVal, InsufficientCapacityError},
        ArrayVec, Drain,
    },
    len::{LengthType, U8},
    mem::{Pattern, SpareMemoryPolicy, Uninitialized},
    testing::dropped::{Dropped, Track},
};
use core::{
    mem,
    ops::{Bound, RangeBounds},
};

struct CustomRange<'a> {
    start: Bound<&'a usize>,
    end: Bound<&'a usize>,
}

impl RangeBounds<usize> for CustomRange<'_> {
    fn start_bound(&self) -> Bound<&usize> {
        self.start
    }
    fn end_bound(&self) -> Bound<&usize> {
        self.end
    }
}

fn check_spare_memory_at<T, L, SM, const C: usize>(
    a: &ArrayVec<T, C, L, SM>,
    pattern: u8,
    start: usize,
    end: usize,
) where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    debug_assert!(start <= end);
    debug_assert!(end <= a.capacity());

    unsafe {
        let mut p = a.as_ptr().add(start) as *const u8;
        let end = a.as_ptr().add(end) as *const u8;

        while p < end {
            assert_eq!(p.read(), pattern);
            p = p.add(1);
        }
    }
}

fn check_spare_memory<T, L, SM, const C: usize>(a: &ArrayVec<T, C, L, SM>, pattern: u8)
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    check_spare_memory_at(a, pattern, a.len(), a.capacity())
}

#[test]
fn test_zst_size() {
    let a = array_vec![3; ()];
    assert_eq!(mem::size_of_val(&a), mem::size_of::<usize>());
}

#[test]
fn test_zst_push_pop() {
    let mut a = array_vec![3; ()];
    while a.has_spare_capacity() {
        a.push(());
    }
    assert_eq!(a.len(), 3);
    assert_eq!(a.spare_capacity(), 0);
    assert!(a.try_push(()).is_err());

    while !a.is_empty() {
        unsafe { a.pop_unchecked() };
    }
    assert_eq!(a.len(), 0);
    assert_eq!(a.spare_capacity(), 3);
}

#[test]
fn test_zst_truncate() {
    let mut a = array_vec![3; ()];
    while a.has_spare_capacity() {
        unsafe {
            a.push_unchecked(());
        }
    }

    assert_eq!(a.len(), 3);

    a.truncate(1);
    assert_eq!(a.len(), 1);

    a.clear();
    assert!(a.is_empty());
}

#[test]
fn test_zst_clone() {
    let mut a = array_vec![3; ()];
    a.push(());
    a.push(());
    assert_eq!(a.len(), 2);

    let b = a.clone();
    assert_eq!(b.len(), 2);
}

#[test]
fn test_capacity_len_empty_full() {
    let mut a = array_vec![2; u64];

    assert_eq!(a.len(), 0);
    assert_eq!(a.capacity(), 2);
    assert_eq!(a.spare_capacity(), a.capacity());
    assert!(a.has_spare_capacity());
    assert!(a.is_empty());
    assert!(!a.is_full());

    a.push(1);
    assert_eq!(a.len(), 1);
    assert!(a.has_spare_capacity());
    assert_eq!(a.spare_capacity(), 1);
    assert!(!a.is_empty());
    assert!(!a.is_full());

    a.push(2);
    assert_eq!(a.len(), a.capacity());
    assert!(!a.has_spare_capacity());
    assert_eq!(a.spare_capacity(), 0);
    assert!(!a.is_empty());
    assert!(a.is_full());
}

#[test]
fn test_push_unchecked() {
    type A = ArrayVec<u16, 5, U8, Pattern<0xBC>>;
    let mut a = A::new();

    assert_eq!(a, []);
    for i in 0..a.capacity() {
        assert_eq!(unsafe { a.as_ptr().add(i).read() }, 0xBCBC);
    }

    unsafe { a.push_unchecked(10) };
    assert_eq!(a, [10]);
    assert_eq!(a.len(), 1);
    assert_eq!(a.spare_capacity(), 4);
    for i in 1..a.capacity() {
        assert_eq!(unsafe { a.as_ptr().add(i).read() }, 0xBCBC);
    }
}

#[test]
fn test_push_unchecked_dropped() {
    type A<'a> = ArrayVec<Dropped<'a, 16>, 16, U8, Pattern<0xBC>>;
    let t = Track::new();
    let mut a = A::from_iter(t.take(3));

    assert!(t.dropped_indices(&[]));

    unsafe { a.push_unchecked(t.alloc()) };

    assert!(t.dropped_indices(&[]));

    drop(a);
    assert!(t.dropped_range(0..=3));
}

#[test]
fn test_try_push() {
    type A = ArrayVec<u16, 3, U8, Pattern<0xBC>>;
    let mut a = A::from_iter(0..2);
    assert_eq!(a, [0, 1]);

    a.try_push(2).expect("try_push failed");
    assert_eq!(a, [0, 1, 2]);

    assert!(matches!(a.try_push(3), Err(e) if e == InsufficientCapacityError));
}

#[test]
fn test_push() {
    type A = ArrayVec<u16, 3, U8, Pattern<0xBC>>;
    let mut a = A::from_iter(0..2);
    assert_eq!(a, [0, 1]);

    for i in 2..a.capacity() {
        assert_eq!(unsafe { a.as_ptr().add(i).read() }, 0xBCBC);
    }

    a.push(2);
    assert_eq!(a, [0, 1, 2]);
}

#[test]
#[should_panic]
fn test_push_panics() {
    type A = ArrayVec<u16, 3, U8, Pattern<0xBC>>;
    let mut a = A::from_iter(0..3);
    assert_eq!(a, [0, 1, 2]);
    assert_eq!(a.spare_capacity(), 0);
    a.push(3);
}

#[test]
fn test_pop() {
    type A = ArrayVec<u64, 2, U8, Pattern<0xAB>>;
    let mut a = A::from_iter(1..3);
    assert_eq!(a.pop(), Some(2));
    assert_eq!(unsafe { a.as_ptr().add(0).read() }, 0x1);
    assert_eq!(unsafe { a.as_ptr().add(1).read() }, 0xABABABABABABABAB);

    assert_eq!(a.pop(), Some(1));
    assert_eq!(unsafe { a.as_ptr().add(0).read() }, 0xABABABABABABABAB);
    assert_eq!(unsafe { a.as_ptr().add(1).read() }, 0xABABABABABABABAB);

    assert_eq!(a.pop(), None);
    assert_eq!(a.pop(), None);
}

#[test]
fn test_pop_dropped() {
    type A<'a> = ArrayVec<Dropped<'a, 16>, 16, U8>;
    let t = Track::new();
    let mut a = A::from_iter(t.take(3));
    assert!(t.dropped_range(0..0)); // empty range
    a.pop();
    assert!(t.dropped_range(2..=2));
    a.pop();
    assert!(t.dropped_range(1..=2));
}

#[test]
fn test_pop_unchecked() {
    type A = ArrayVec<u8, 4, U8, Pattern<0xAB>>;
    let mut a = A::from_iter(5..9);
    assert_eq!(unsafe { a.pop_unchecked() }, 8);
    assert_eq!(unsafe { a.pop_unchecked() }, 7);
    assert_eq!(unsafe { a.as_ptr().add(0).read() }, 5);
    assert_eq!(unsafe { a.as_ptr().add(1).read() }, 6);
    assert_eq!(unsafe { a.as_ptr().add(2).read() }, 0xAB);
    assert_eq!(unsafe { a.as_ptr().add(3).read() }, 0xAB);
}

#[test]
fn test_pop_unchecked_dropped() {
    type A<'a> = ArrayVec<Dropped<'a, 16>, 16, U8>;
    let t = Track::new();
    let mut a = A::from_iter(t.take(3));
    assert!(t.dropped_range(0..0)); // empty range
    unsafe { a.pop_unchecked() };
    assert!(t.dropped_range(2..=2));
    unsafe { a.pop_unchecked() };
    assert!(t.dropped_range(1..=2));
}

#[test]
fn test_remove_unchecked() {
    type A = ArrayVec<u8, 5, U8, Pattern<0xAB>>;
    let mut a = A::from_iter(1..6);
    assert_eq!(a, [1, 2, 3, 4, 5]);
    assert_eq!(a.len(), 5);
    assert_eq!(a.spare_capacity(), 0);

    assert_eq!(unsafe { a.remove_unchecked(0) }, 1);
    assert_eq!(a, [2, 3, 4, 5]);
    assert_eq!(a.len(), 4);
    assert_eq!(a.spare_capacity(), 1);
    assert_eq!(unsafe { a.as_ptr().add(4).read() }, 0xAB);

    assert_eq!(unsafe { a.remove_unchecked(1) }, 3);
    assert_eq!(a, [2, 4, 5]);
    assert_eq!(a.len(), 3);
    assert_eq!(a.spare_capacity(), 2);
    assert_eq!(unsafe { a.as_ptr().add(3).read() }, 0xAB);
    assert_eq!(unsafe { a.as_ptr().add(4).read() }, 0xAB);
}

#[test]
fn test_remove_unchecked_dropped() {
    type A<'a> = ArrayVec<Dropped<'a, 16>, 16, U8>;
    let t = Track::new();
    let mut a = A::from_iter(t.take(5));
    assert!(t.dropped_range(0..0)); // empty range

    unsafe { a.remove_unchecked(1) };
    assert!(t.dropped_range(1..=1));

    unsafe { a.remove_unchecked(1) };
    assert!(t.dropped_range(1..=2));
}

#[test]
fn test_remove() {
    type A = ArrayVec<u8, 5, U8, Pattern<0xAB>>;
    let mut a = A::from_iter(1..6);
    assert_eq!(a, [1, 2, 3, 4, 5]);

    assert_eq!(a.remove(0), 1);
    assert_eq!(a, [2, 3, 4, 5]);
    assert_eq!(unsafe { a.as_ptr().add(4).read() }, 0xAB);

    assert_eq!(a.remove(1), 3);
    assert_eq!(a, [2, 4, 5]);
    assert_eq!(unsafe { a.as_ptr().add(3).read() }, 0xAB);
    assert_eq!(unsafe { a.as_ptr().add(4).read() }, 0xAB);

    assert_eq!(a.remove(2), 5);
    assert_eq!(a, [2, 4]);
    assert_eq!(unsafe { a.as_ptr().add(2).read() }, 0xAB);
    assert_eq!(unsafe { a.as_ptr().add(3).read() }, 0xAB);
    assert_eq!(unsafe { a.as_ptr().add(4).read() }, 0xAB);
}

#[test]
#[should_panic]
fn test_remove_invalid_index() {
    let mut a = array_vec![3; u64; 1];
    a.remove(1);
}

#[test]
fn test_remove_dropped() {
    type A<'a> = ArrayVec<Dropped<'a, 16>, 16, U8>;
    let t = Track::new();
    let mut a = A::from_iter(t.take(5));
    assert!(t.dropped_range(0..0)); // empty range

    a.remove(1);
    assert!(t.dropped_range(1..=1));

    a.remove(1);
    assert!(t.dropped_range(1..=2));

    assert_eq!(a.len(), 3);
    assert_eq!(a.spare_capacity(), 13);
}

#[test]
fn test_try_remove() {
    type A = ArrayVec<u16, 5, U8, Pattern<0xAB>>;
    let mut a = A::from_iter(1..4);

    assert_eq!(a, [1, 2, 3]);
    for i in 3..a.capacity() {
        assert_eq!(unsafe { a.as_ptr().add(i).read() }, 0xABAB);
    }

    assert_eq!(a.try_remove(0), Some(1));
    assert_eq!(a.try_remove(1), Some(3));
    assert_eq!(a.try_remove(1), None);
    assert_eq!(a.try_remove(0), Some(2));

    for i in 0..a.capacity() {
        assert_eq!(unsafe { a.as_ptr().add(i).read() }, 0xABAB);
    }
}

#[test]
fn test_try_remove_dropped() {
    type A<'a> = ArrayVec<Dropped<'a, 16>, 16, U8>;
    let t = Track::new();
    let mut a = A::from_iter(t.take(5));
    assert!(t.dropped_range(0..0)); // empty range

    for i in 1..=4 {
        a.try_remove(1);
        assert!(t.dropped_range(1..=i));
    }

    assert_eq!(a.len(), 1);
    assert_eq!(a.spare_capacity(), 15);
}

#[test]
fn test_try_from_iter() {
    type A = ArrayVec<usize, 5, U8>;

    let a = A::try_from_iter(0..5).expect("try_from_iter failed");
    assert_eq!(a, [0, 1, 2, 3, 4]);

    assert!(matches!(
        A::try_from_iter(0..A::CAPACITY + 1),
        Err(e) if e == InsufficientCapacityError
    ));
}

#[test]
fn test_iter() {
    let a = array_vec![5; u16; 1, 2, 3];
    for (i, a) in a.iter().enumerate() {
        assert_eq!(*a, (i + 1) as u16);
    }
}

#[test]
fn test_iter_mut() {
    let mut a = array_vec![5; u16; 1, 2, 3];
    for e in a.iter_mut() {
        *e *= 2;
    }
    assert_eq!(a, [2, 4, 6]);
}

#[test]
fn test_insert_unchecked() {
    type A = ArrayVec<u8, 5, U8, Pattern<0xAB>>;
    let mut a = A::from_iter(0..4);
    assert_eq!(a, [0, 1, 2, 3]);
    assert_eq!(unsafe { a.as_ptr().add(4).read() }, 0xAB);

    unsafe { a.insert_unchecked(1, 5) };
    assert_eq!(a, [0, 5, 1, 2, 3]);
}

#[test]
fn test_insert_unchecked_dropped() {
    type A<'a> = ArrayVec<Dropped<'a, 16>, 16, U8>;
    let t = Track::new();
    let mut a = A::from_iter(t.take(3));
    assert!(t.dropped_range(0..0)); // empty range

    unsafe { a.insert_unchecked(1, t.alloc()) };

    a.pop();
    a.pop();

    assert!(t.dropped_range(1..=2));

    a.remove(0);

    assert!(t.dropped_range(0..=2));
}

#[test]
fn test_try_insert() {
    type A = ArrayVec<u8, 5, U8, Pattern<0xAB>>;
    let mut a = A::from_iter(0..3);
    assert_eq!(a, [0, 1, 2]);
    assert_eq!(unsafe { a.as_ptr().add(3).read() }, 0xAB);
    assert_eq!(unsafe { a.as_ptr().add(4).read() }, 0xAB);

    assert!(matches!(a.try_insert(4, 4), Err(InsertError::InvalidIndex)));

    a.try_insert(1, 4).expect("try_insert failed");
    assert_eq!(a, [0, 4, 1, 2]);

    a.try_insert(4, 5).expect("try_insert failed");
    assert_eq!(a, [0, 4, 1, 2, 5]);

    assert!(matches!(
        a.try_insert(1, 8),
        Err(InsertError::InsufficientCapacity)
    ));
}

#[test]
fn test_insert() {
    type A = ArrayVec<u8, 5, U8, Pattern<0xAB>>;
    let mut a = A::from_iter(0..3);
    assert_eq!(a, [0, 1, 2]);
    assert_eq!(unsafe { a.as_ptr().add(3).read() }, 0xAB);
    assert_eq!(unsafe { a.as_ptr().add(4).read() }, 0xAB);

    a.insert(0, 0);
    assert_eq!(a, [0, 0, 1, 2]);
    assert_eq!(unsafe { a.as_ptr().add(4).read() }, 0xAB);

    a.insert(4, 3);
    assert_eq!(a, [0, 0, 1, 2, 3]);
}

#[test]
#[should_panic]
fn test_insert_panics() {
    type A = ArrayVec<u8, 5, U8, Pattern<0xAB>>;
    let mut a = A::from_iter(0..5);
    a.insert(0, 0);
}

#[test]
fn test_swap_remove_unchecked() {
    type A = ArrayVec<u8, 5, U8, Pattern<0xAC>>;
    let mut a = A::from_iter(0..4);
    assert_eq!(a, [0, 1, 2, 3]);
    assert_eq!(unsafe { a.as_ptr().add(4).read() }, 0xAC);

    assert_eq!(unsafe { a.swap_remove_unchecked(1) }, 1);
    assert_eq!(a, [0, 3, 2]);
    assert_eq!(unsafe { a.as_ptr().add(3).read() }, 0xAC);
    assert_eq!(unsafe { a.as_ptr().add(4).read() }, 0xAC);

    assert_eq!(unsafe { a.swap_remove_unchecked(2) }, 2);
    assert_eq!(a, [0, 3]);
    assert_eq!(unsafe { a.as_ptr().add(2).read() }, 0xAC);
    assert_eq!(unsafe { a.as_ptr().add(3).read() }, 0xAC);
    assert_eq!(unsafe { a.as_ptr().add(4).read() }, 0xAC);
}

#[test]
fn test_swap_remove_unchecked_dropped() {
    type A<'a> = ArrayVec<Dropped<'a, 16>, 16, U8, Pattern<0xAC>>;
    let t = Track::new();
    let mut a = A::from_iter(t.take(5));
    assert!(t.dropped_indices(&[]));

    unsafe { a.swap_remove_unchecked(1) };
    assert!(t.dropped_indices(&[1]));

    unsafe { a.swap_remove_unchecked(1) };
    assert!(t.dropped_indices(&[1, 4]));
}

#[test]
fn test_try_swap_remove() {
    type A = ArrayVec<u8, 5, U8, Pattern<0xAC>>;
    let mut a = A::from_iter(0..5);
    assert_eq!(a, [0, 1, 2, 3, 4]);

    assert_eq!(a.try_swap_remove(1), Some(1));
    assert_eq!(a, [0, 4, 2, 3]);
    assert_eq!(unsafe { a.as_ptr().add(4).read() }, 0xAC);

    assert_eq!(a.try_swap_remove(1), Some(4));
    assert_eq!(a, [0, 3, 2]);
    assert_eq!(unsafe { a.as_ptr().add(3).read() }, 0xAC);
    assert_eq!(unsafe { a.as_ptr().add(4).read() }, 0xAC);

    assert_eq!(a.try_swap_remove(10), None);
    assert_eq!(a, [0, 3, 2]);
}

#[test]
fn test_try_swap_remove_dropped() {
    type A<'a> = ArrayVec<Dropped<'a, 5>, 5, U8, Pattern<0xAC>>;
    let t = Track::new();
    let mut a = A::from_iter(t.take(4));
    assert!(t.dropped_indices(&[]));

    assert!(a.try_swap_remove(1).is_some());
    assert!(t.dropped_indices(&[1]));

    assert!(a.try_swap_remove(1).is_some());
    assert!(t.dropped_indices(&[1, 3]));

    assert!(a.try_swap_remove(10).is_none());
}

#[test]
fn test_swap_remove() {
    type A = ArrayVec<u8, 5, U8, Pattern<0xAC>>;
    let mut a = A::from_iter(0..5);
    assert_eq!(a, [0, 1, 2, 3, 4]);

    assert_eq!(a.swap_remove(1), 1);
    assert_eq!(a, [0, 4, 2, 3]);
    assert_eq!(unsafe { a.as_ptr().add(4).read() }, 0xAC);

    assert_eq!(a.swap_remove(1), 4);
    assert_eq!(a, [0, 3, 2]);
    assert_eq!(unsafe { a.as_ptr().add(3).read() }, 0xAC);
    assert_eq!(unsafe { a.as_ptr().add(4).read() }, 0xAC);
}

#[test]
fn test_swap_remove_dropped() {
    type A<'a> = ArrayVec<Dropped<'a, 5>, 5, U8, Pattern<0xAC>>;
    let t = Track::new();
    let mut a = A::from_iter(t.take(4));
    assert!(t.dropped_indices(&[]));

    a.swap_remove(1);
    assert!(t.dropped_indices(&[1]));

    a.swap_remove(1);
    assert!(t.dropped_indices(&[1, 3]));
}

#[test]
#[should_panic]
fn test_swap_remove_panics() {
    type A = ArrayVec<u8, 5, U8, Pattern<0xAC>>;
    let mut a = A::from_iter(0..1);
    assert_eq!(a, [0]);
    a.swap_remove(2);
}

#[test]
fn test_try_push_val() {
    type A = ArrayVec<u8, 4, U8>;
    let mut a = A::from_iter(0..3);
    assert_eq!(a, [0, 1, 2]);

    assert!(a.try_push_val(3).is_ok());
    assert_eq!(a, [0, 1, 2, 3]);

    let res = a.try_push_val(4);
    assert!(res.is_err());
    assert_eq!(4, res.unwrap_err().0);
    assert_eq!(a, [0, 1, 2, 3]);
}

#[test]
fn test_try_push_val_dropped() {
    type A<'a> = ArrayVec<Dropped<'a, 5>, 4, U8, Pattern<0xAC>>;
    let t = Track::new();
    let mut a = A::from_iter(t.take(3));
    assert!(t.dropped_indices(&[]));

    assert!(a.try_push_val(t.alloc()).is_ok());
    assert!(t.dropped_indices(&[]));

    let v = t.alloc();
    let res = a.try_push_val(v);
    assert!(res.is_err());
    assert!(t.dropped_indices(&[]));

    drop(res);
    assert!(t.dropped_indices(&[4]));

    drop(a);
    assert!(t.dropped_range(0..=4));
}

#[test]
fn test_try_insert_val() {
    type A = ArrayVec<u8, 4, U8, Uninitialized>;
    let mut a = A::from_iter(0..3);
    assert_eq!(a, [0, 1, 2]);

    let res = a.try_insert_val(4, 14);
    assert!(matches!(res, Err(InsertErrorVal::InvalidIndex(v)) if v == 14));
    assert_eq!(res.unwrap_err().into_value(), 14);

    assert!(a.try_insert_val(1, 7).is_ok());
    assert_eq!(a, [0, 7, 1, 2]);

    let res = a.try_insert_val(0, 14);
    assert!(matches!(res, Err(InsertErrorVal::InsufficientCapacity(v)) if v == 14));
    assert_eq!(a, [0, 7, 1, 2]);
}

#[test]
fn test_try_insert_val_dropped() {
    type A<'a> = ArrayVec<Dropped<'a, 10>, 4, U8>;
    let t = Track::new();
    let mut a = A::from_iter(t.take(3));
    assert!(t.dropped_indices(&[]));

    assert!(a.try_insert_val(5, t.alloc()).is_err());
    assert!(t.dropped_indices(&[3]));

    assert!(a.try_insert_val(1, t.alloc()).is_ok());
    assert!(t.dropped_indices(&[3]));

    let res = a.try_insert_val(0, t.alloc());
    assert!(matches!(
        res,
        Err(InsertErrorVal::InsufficientCapacity(ref _v))
    ));
    assert!(t.dropped_indices(&[3]));

    drop(res);
    assert!(t.dropped_indices(&[3, 5]));
}

#[test]
fn test_drain_empty_range() {
    let mut a = array_vec![3; 1, 2, 3];
    assert_eq!(a, [1, 2, 3]);
    {
        let d = a.drain(0..0);
        assert_eq!(d.len(), 0);
    }
    assert_eq!(a, [1, 2, 3]);
}

#[test]
fn test_drain_unbounded_end() {
    let mut a = array_vec![3; 1, 2, 3];
    assert_eq!(a, [1, 2, 3]);
    {
        let d = a.drain(1..);
        assert_eq!(d.len(), 2);
    }
    assert_eq!(a, [1]);
}

#[test]
fn test_drain_unbounded_start() {
    let mut a = array_vec![3; 1, 2, 3];
    assert_eq!(a, [1, 2, 3]);
    {
        let d = a.drain(..2);
        assert_eq!(d.len(), 2);
    }
    assert_eq!(a, [3]);
}

#[test]
fn test_drain_unbounded_range() {
    let mut a = array_vec![3; 1, 2, 3];
    assert_eq!(a, [1, 2, 3]);

    for (i, e) in a.drain(..).enumerate() {
        assert_eq!(i + 1, e);
    }

    assert_eq!(a, []);
}

#[test]
fn test_drain_included_end() {
    let mut a = array_vec![3; 1, 2, 3];
    assert_eq!(a, [1, 2, 3]);
    {
        let d = a.drain(0..=1);
        assert_eq!(d.len(), 2);
    }
    assert_eq!(a, [3]);
}

#[test]
fn test_drain_excluded_start() {
    let mut a = array_vec![3; 1, 2, 3];
    assert_eq!(a, [1, 2, 3]);

    let start: usize = 0;
    let end: usize = 1;

    let r = CustomRange {
        start: Bound::Excluded(&start),
        end: Bound::Included(&end),
    };

    for (i, e) in a.drain(r).enumerate() {
        assert_eq!(i + 2, e);
    }

    assert_eq!(a, [1, 3]);
}

#[test]
#[should_panic]
fn test_drain_start_overflow() {
    let start = usize::MAX;
    let end = 1usize;

    let r = CustomRange {
        start: Bound::Excluded(&start),
        end: Bound::Included(&end),
    };

    let mut a = array_vec![3; 1, 2, 3];
    a.drain(r);
}

#[test]
#[should_panic]
fn test_drain_end_overflow() {
    let mut a = array_vec![3; 1, 2, 3];
    a.drain(0..=usize::MAX);
}

#[test]
#[should_panic]
fn test_drain_end_out_of_bounds() {
    let mut a = array_vec![3; 1, 2, 3];
    a.drain(0..=3);
}

#[test]
#[should_panic]
fn test_drain_invalid_range() {
    let mut a = array_vec![3; 1, 2, 3];
    #[allow(clippy::reversed_empty_ranges)]
    a.drain(1..0);
}

#[test]
fn test_drain_prefix_smp() {
    type A = ArrayVec<usize, 5, U8, Pattern<0xAB>>;
    const SPARE_MEM: usize = 0xABABABABABABABAB;

    let mut a = A::from_iter(0..5);
    assert_eq!(a, [0, 1, 2, 3, 4]);
    assert_eq!(a.len(), 5);

    {
        let mut d = a.drain(0..2);
        assert_eq!(d.len(), 2);
        assert_eq!(d.next(), Some(0));
        assert_eq!(d.next(), Some(1));
        assert_eq!(d.next(), None);
    }

    assert_eq!(a.len(), 3);
    assert_eq!(a, [2, 3, 4]);
    assert_eq!(unsafe { a.as_ptr().add(3).read() }, SPARE_MEM);
    assert_eq!(unsafe { a.as_ptr().add(4).read() }, SPARE_MEM);

    // do not consume the whole iterator
    a.drain(0..1);
    assert_eq!(a, [3, 4]);
    assert_eq!(unsafe { a.as_ptr().add(2).read() }, SPARE_MEM);
    assert_eq!(unsafe { a.as_ptr().add(3).read() }, SPARE_MEM);
    assert_eq!(unsafe { a.as_ptr().add(4).read() }, SPARE_MEM);
}

#[test]
fn test_drain_suffix_smp() {
    type A = ArrayVec<usize, 5, U8, Pattern<0xAB>>;
    const SPARE_MEM: usize = 0xABABABABABABABAB;

    let mut a = A::from_iter(0..5);
    assert_eq!(a, [0, 1, 2, 3, 4]);
    assert_eq!(a.len(), 5);

    {
        let mut d = a.drain(3..5);
        assert_eq!(d.len(), 2);
        assert_eq!(d.next(), Some(3));
        assert_eq!(d.next(), Some(4));
        assert_eq!(d.next(), None);
    }

    assert_eq!(a.len(), 3);
    assert_eq!(a, [0, 1, 2]);
    assert_eq!(unsafe { a.as_ptr().add(3).read() }, SPARE_MEM);
    assert_eq!(unsafe { a.as_ptr().add(4).read() }, SPARE_MEM);

    // do not consume the whole iterator
    a.drain(2..3);
    assert_eq!(a, [0, 1]);
    assert_eq!(unsafe { a.as_ptr().add(2).read() }, SPARE_MEM);
    assert_eq!(unsafe { a.as_ptr().add(3).read() }, SPARE_MEM);
    assert_eq!(unsafe { a.as_ptr().add(4).read() }, SPARE_MEM);
}

#[test]
fn test_drain_zst() {
    type A = ArrayVec<(), 5, U8>;
    let mut a = A::new();
    while a.has_spare_capacity() {
        a.push(());
    }
    assert_eq!(a.len(), a.capacity());

    #[allow(clippy::unit_cmp)]
    for e in a.drain(1..3) {
        assert_eq!(e, ());
    }
    assert_eq!(a.len(), a.capacity() - 2);
}

#[test]
fn test_drain_dropped() {
    type A<'a> = ArrayVec<Dropped<'a, 16>, 16, U8>;
    let t = Track::<16>::new();
    let mut a = A::from_iter(t.take(5));
    assert!(t.dropped_indices(&[]));

    a.drain(1..3);
    assert!(t.dropped_range(1..3));
}

#[test]
fn test_extend() {
    let mut a = array_vec![5; 1, 2, 3];
    assert_eq!(a, [1, 2, 3]);
    a.extend(4..6);
    assert_eq!(a, [1, 2, 3, 4, 5]);
}

#[test]
#[should_panic]
fn test_extend_panics() {
    let mut a = array_vec![2; 1, 2];
    a.extend(0..1);
}

#[test]
fn test_retain() {
    let mut a = array_vec![5; 1, 2, 3, 4, 5];
    assert_eq!(a, [1, 2, 3, 4, 5]);
    a.retain(|e| *e < 4);
    assert_eq!(a, [1, 2, 3]);
}

#[test]
fn test_retain_mut() {
    let mut a = array_vec![5; 1, 2, 3, 4, 5];
    assert_eq!(a, [1, 2, 3, 4, 5]);
    a.retain_mut(|e| {
        *e *= 2;
        *e > 5
    });
    assert_eq!(a, [6, 8, 10]);
}

#[test]
fn test_retain_dropped() {
    type A<'a> = ArrayVec<Dropped<'a, 16>, 8, U8, Pattern<0xBA>>;
    let t = Track::<16>::new();
    let mut a = A::from_iter(t.take(A::CAPACITY));
    assert!(t.dropped_indices(&[]));
    a.retain(|e| e.idx() < 2);
    assert!(t.dropped_range(2..8));
    check_spare_memory_at(&a, 0xBA, 2, 8);
    drop(a);
    assert!(t.dropped_range(0..8));
}

#[test]
fn test_retain_dropped_retain_all() {
    type A<'a> = ArrayVec<Dropped<'a, 16>, 8, U8, Pattern<0xBA>>;
    let t = Track::<16>::new();
    let mut a = A::from_iter(t.take(A::CAPACITY));
    assert_eq!(a.len(), 8);
    assert!(t.dropped_range(0..0));
    a.retain(|_| true);
    assert!(t.dropped_range(0..0));
    assert_eq!(a.len(), 8);
    drop(a);
    assert!(t.dropped_range(0..8));
}

#[test]
fn test_retain_dropped_retain_none() {
    type A<'a> = ArrayVec<Dropped<'a, 16>, 8, U8, Pattern<0xBA>>;
    let t = Track::<16>::new();
    let mut a = A::from_iter(t.take(A::CAPACITY));
    assert_eq!(a.len(), 8);
    assert!(t.dropped_range(0..0));
    a.retain(|_| false);
    assert_eq!(a.len(), 0);
    check_spare_memory(&a, 0xBA);
    assert!(t.dropped_range(0..8));
}

#[test]
fn test_spare_capacity_mut_dropped() {
    type A<'a> = ArrayVec<Dropped<'a, 16>, 8, U8, Pattern<0xBA>>;
    let t = Track::<16>::new();
    let mut a = A::new();

    let spare = a.spare_capacity_mut();
    spare[0].write(t.alloc());
    spare[1].write(t.alloc());

    unsafe { a.set_len(2) };

    assert!(t.dropped_range(0..0));
    drop(a);
    assert!(t.dropped_range(0..2));
}

#[test]
fn test_split_at_spare_mut_dropped() {
    type A<'a> = ArrayVec<Dropped<'a, 16>, 8, U8, Pattern<0xBA>>;
    let t = Track::<16>::new();
    let mut a = A::from_iter(t.take(2));
    assert!(t.dropped_range(0..0));

    let (init, spare) = a.split_at_spare_mut();
    assert_eq!(init[0].idx(), 0);
    assert_eq!(init[1].idx(), 1);

    spare[0].write(t.alloc());
    spare[1].write(t.alloc());

    unsafe { a.set_len(a.len() + 2) };

    assert!(t.dropped_range(0..0));
    drop(a);
    assert!(t.dropped_range(0..4));
}

#[test]
fn test_resize() {
    let mut av = array_vec![5; 1, 2];
    assert_eq!(av, [1, 2]);

    av.resize(2, 0);
    assert_eq!(av, [1, 2]);

    av.resize(3, 0);
    assert_eq!(av, [1, 2, 0]);

    av.resize(av.capacity(), 7);
    assert_eq!(av, [1, 2, 0, 7, 7]);

    av.resize(1, 2);
    assert_eq!(av, [1]);

    av.resize(0, 2);
    assert_eq!(av, []);
}

#[test]
#[should_panic]
fn test_resize_panics() {
    let mut av = array_vec![5; 1, 2];
    av.resize(av.capacity() + 1, 0);
}

#[test]
fn test_try_resize() {
    let mut av = array_vec![5; 1, 2];
    assert_eq!(av, [1, 2]);

    av.try_resize(4, 7).unwrap();
    assert_eq!(av, [1, 2, 7, 7]);

    // same new_len as before
    av.try_resize(4, 0).unwrap();
    assert_eq!(av, [1, 2, 7, 7]);

    av.try_resize(0, 0).unwrap();
    assert_eq!(av, []);

    assert!(matches!(av.try_resize(10, 0), Err(e) if e == InsufficientCapacityError));
}

#[test]
fn test_resize_with_dropped() {
    type A<'a> = ArrayVec<Dropped<'a, 16>, 8, U8, Pattern<0xBA>>;
    let t = Track::<16>::new();
    let mut av = A::from_iter(t.take(2));
    assert!(t.dropped_range(0..0));

    av.resize_with(av.capacity(), || t.alloc());
    assert_eq!(av.len(), av.capacity());
    assert!(t.dropped_range(0..0));

    av.resize_with(1, || t.alloc());
    assert_eq!(av.len(), 1);
    assert!(t.dropped_range(1..8));
}

#[test]
#[should_panic]
fn test_resize_with_panics() {
    let mut av = array_vec![5; 1, 2];
    av.resize_with(av.capacity() + 1, || 0);
}

#[test]
fn test_try_resize_with_dropped() {
    type A<'a> = ArrayVec<Dropped<'a, 16>, 8, U8, Pattern<0xBA>>;
    let t = Track::<16>::new();
    let mut av = A::from_iter(t.take(2));
    assert!(t.dropped_range(0..0));

    av.resize_with(av.capacity(), || t.alloc());
    assert_eq!(av.len(), av.capacity());
    assert!(t.dropped_range(0..0));

    av.resize_with(0, || t.alloc());
    assert_eq!(av.len(), 0);
    assert!(t.dropped_range(0..8));
}

#[test]
fn test_copy_from_slice() {
    let mut av = array_vec![5;];
    assert_eq!(av, []);
    av.copy_from_slice(&[1, 2]);
    assert_eq!(av, [1, 2]);
    av.copy_from_slice(&[3, 4]);
    assert_eq!(av, [1, 2, 3, 4]);
    av.copy_from_slice(&[5]);
    assert_eq!(av, [1, 2, 3, 4, 5]);
}

#[test]
#[should_panic]
fn test_copy_from_slice_panics() {
    let mut av = array_vec![3; 1, 2];
    assert_eq!(av, [1, 2]);
    av.copy_from_slice(&[3, 4]);
}

#[test]
fn test_try_copy_from_slice() {
    let mut av = array_vec![5;];
    assert_eq!(av, []);
    av.try_copy_from_slice(&[1, 2]).unwrap();
    assert_eq!(av, [1, 2]);
    av.try_copy_from_slice(&[3, 4]).unwrap();
    assert_eq!(av, [1, 2, 3, 4]);
    av.try_copy_from_slice(&[5]).unwrap();
    assert_eq!(av, [1, 2, 3, 4, 5]);
    assert!(matches!(av.try_copy_from_slice(&[6]), Err(e) if e == InsufficientCapacityError));
}

#[test]
fn test_copy_from_slice_unchecked() {
    let mut av = array_vec![5;];
    unsafe { av.copy_from_slice_unchecked(&[1, 2, 3]) };
    assert_eq!(av, [1, 2, 3]);
}

#[test]
#[allow(dead_code)]
fn test_arrayvec_covariance() {
    fn foo<'a>(av: ArrayVec<&'static str, 8>) -> ArrayVec<&'a str, 8> {
        av
    }
}

#[test]
#[allow(dead_code)]
fn test_arrayvec_drain_covariance() {
    fn foo<'a>(
        d: Drain<'static, &'static str, U8, Uninitialized, 8>,
    ) -> Drain<'a, &'a str, U8, Uninitialized, 8> {
        d
    }
}
