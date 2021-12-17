use crate as cds;
use cds::{
    array_vec,
    arrayvec::ArrayVec,
    defs::{Pattern, Uninitialized, U8},
    testing::dropped::{Dropped, Track},
};
use core::mem;

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
    assert_eq!(a.try_push(()).is_err(), true);

    while !a.is_empty() {
        let _zst = unsafe { a.pop_unchecked() };
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
    assert_eq!(a.has_spare_capacity(), true);
    assert_eq!(a.is_empty(), true);
    assert_eq!(a.is_full(), false);

    a.push(1);
    assert_eq!(a.len(), 1);
    assert_eq!(a.has_spare_capacity(), true);
    assert_eq!(a.spare_capacity(), 1);
    assert_eq!(a.is_empty(), false);
    assert_eq!(a.is_full(), false);

    a.push(2);
    assert_eq!(a.len(), a.capacity());
    assert_eq!(a.has_spare_capacity(), false);
    assert_eq!(a.spare_capacity(), 0);
    assert_eq!(a.is_empty(), false);
    assert_eq!(a.is_full(), true);
}

#[test]
fn test_pop() {
    type A = ArrayVec<u64, U8, Pattern<0xAB>, 2>;
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
    type A<'a> = ArrayVec<Dropped<'a, 16>, U8, Uninitialized, 16>;
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
    type A = ArrayVec<u8, U8, Pattern<0xAB>, 4>;
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
    type A<'a> = ArrayVec<Dropped<'a, 16>, U8, Uninitialized, 16>;
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
    type A = ArrayVec<u8, U8, Pattern<0xAB>, 5>;
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
    type A<'a> = ArrayVec<Dropped<'a, 16>, U8, Uninitialized, 16>;
    let t = Track::new();
    let mut a = A::from_iter(t.take(5));
    assert!(t.dropped_range(0..0)); // empty range

    unsafe { a.remove_unchecked(1) };
    assert!(t.dropped_range(1..=1));

    unsafe { a.remove_unchecked(1) };
    assert!(t.dropped_range(1..=2));
}
