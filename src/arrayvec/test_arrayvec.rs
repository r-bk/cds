use crate as cds;
use crate::array_vec;
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
    assert_eq!(a.spare_capacity_len(), 0);
    assert_eq!(a.try_push(()).is_err(), true);

    while !a.is_empty() {
        let _zst = unsafe { a.pop_unchecked() };
    }
    assert_eq!(a.len(), 0);
    assert_eq!(a.spare_capacity_len(), 3);
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
