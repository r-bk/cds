use super::*;
use core::mem;

#[derive(Debug, Clone)]
struct ZST;

#[test]
fn test_zst_size() {
    let a = ArrayVec::<ZST, 3>::new();
    assert_eq!(mem::size_of_val(&a), mem::size_of::<usize>());
}

#[test]
fn test_zst_push_pop() {
    let mut a = ArrayVec::<ZST, 3>::new();
    while a.has_spare_capacity() {
        a.push(ZST {});
    }
    assert_eq!(a.len(), 3);
    assert_eq!(a.spare_capacity_len(), 0);
    assert_eq!(a.try_push(ZST {}).is_err(), true);

    while !a.is_empty() {
        let _zst = unsafe { a.pop_unchecked() };
    }
    assert_eq!(a.len(), 0);
    assert_eq!(a.spare_capacity_len(), 3);
}

#[test]
fn test_zst_truncate() {
    let mut a = ArrayVec::<ZST, 3>::new();
    while a.has_spare_capacity() {
        unsafe {
            a.push_unchecked(ZST {});
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
    let mut a = ArrayVec::<ZST, 3>::new();
    a.push(ZST {});
    a.push(ZST {});
    assert_eq!(a.len(), 2);

    let b = a.clone();
    assert_eq!(b.len(), 2);
}
