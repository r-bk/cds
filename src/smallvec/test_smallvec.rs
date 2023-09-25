use crate as cds;
use cds::{
    gen_dropped_zst,
    len::{LengthType, Usize, U8},
    mem::{errors::ReservationError, Pattern, SpareMemoryPolicy, Uninitialized},
    small_vec,
    smallvec::{errors::InsertError, Drain, SmallVec},
    testing::{
        dropped::{Dropped, Track},
        dropped_zst::{counters, Counters},
    },
};
use core::{
    ops::{Bound, RangeBounds},
    ptr,
};

fn check_spare_memory_at<T, L, SM, const C: usize>(
    v: &SmallVec<T, C, L, SM>,
    pattern: u8,
    start: usize,
    end: usize,
) where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    debug_assert!(start <= end);
    debug_assert!(end <= v.capacity());

    unsafe {
        let mut p = v.as_ptr().add(start) as *const u8;
        let end = v.as_ptr().add(end) as *const u8;

        while p < end {
            assert_eq!(p.read(), pattern);
            p = p.add(1);
        }
    }
}

fn check_spare_memory<T, L, SM, const C: usize>(v: &SmallVec<T, C, L, SM>, pattern: u8)
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    check_spare_memory_at(v, pattern, v.len(), v.capacity())
}

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

#[test]
fn test_zst_capacity() {
    let v = SmallVec::<(), 32, U8>::new();
    assert_eq!(v.capacity(), u8::MAX as usize);
    assert!(!v.is_heap());
}

#[test]
fn test_zst_never_heap() {
    let mut v = SmallVec::<(), 16, U8>::new();
    for _ in 0..32 {
        v.push(());
    }
    assert_eq!(v.len(), 32);
    assert!(!v.is_heap());
}

#[test]
fn test_with_capacity() {
    type SV = SmallVec<usize, 16, U8>;
    let v = SV::with_capacity(8);
    assert_eq!(v.capacity(), 16);
    assert!(!v.is_heap());
    let v = SV::with_capacity(16);
    assert_eq!(v.capacity(), 16);
    assert!(!v.is_heap());
    let v = SV::with_capacity(17);
    assert_eq!(v.capacity(), 17);
    assert!(v.is_heap());
}

#[test]
fn test_as_slice() {
    type SV = SmallVec<usize, 3, U8>;
    let mut v = SV::try_from([1, 2, 3]).unwrap();
    assert!(!v.is_heap());
    assert_eq!(v.as_slice(), &[1, 2, 3]);
    assert_eq!(v.as_mut_slice(), &[1, 2, 3]);

    v.push(4);
    assert!(v.is_heap());
    assert_eq!(v.as_slice(), &[1, 2, 3, 4]);
    assert_eq!(v.as_mut_slice(), &[1, 2, 3, 4]);
}

#[test]
fn test_as_ptr() {
    let mut v = small_vec![3; usize];
    assert!(v.is_local());
    assert_eq!(v.as_ptr().cast(), ptr::addr_of!(v));
    assert_eq!(v.as_mut_ptr().cast(), ptr::addr_of_mut!(v));
    assert_eq!(v.as_ptr(), v.as_mut_ptr());

    v.reserve(10);
    assert!(!v.is_local());
    assert_ne!(v.as_ptr().cast(), ptr::addr_of!(v));
    assert_ne!(v.as_mut_ptr().cast(), ptr::addr_of_mut!(v));
    assert_eq!(v.as_ptr(), v.as_mut_ptr());
}

#[test]
fn test_clear() {
    const P: u8 = 0xBC;
    type SV = SmallVec<usize, 3, U8, Pattern<P>>;

    let mut v = SV::try_from([10, 12, 15]).unwrap();
    assert_eq!(v, [10, 12, 15]);
    check_spare_memory(&v, P);
    v.clear();
    assert_eq!(v, []);
    assert!(v.is_empty());
    assert!(!v.is_heap());
    check_spare_memory(&v, P);

    let mut v = SV::try_from([10, 12, 15, 19]).unwrap();
    assert_eq!(v, [10, 12, 15, 19]);
    check_spare_memory(&v, P);
    v.clear();
    assert_eq!(v, []);
    assert!(v.is_empty());
    assert!(v.is_heap());
    check_spare_memory(&v, P);
}

fn test_reserve_impl<const C: usize, L, SM>(v: &mut SmallVec<usize, C, L, SM>, pattern: u8)
where
    L: LengthType,
    SM: SpareMemoryPolicy<usize>,
{
    if !SM::NOOP {
        check_spare_memory(v, pattern);
    }
    assert!(v.is_local());

    for i in 0..4 {
        v.reserve(1);
        assert!(v.is_local());
        assert_eq!(v.capacity(), 4);
        assert_eq!(v.len(), i);
        if !SM::NOOP {
            check_spare_memory(v, pattern);
        }
        v.push(1);
    }

    v.reserve(1);
    assert!(!v.is_local());
    assert_eq!(v.capacity(), 8);
    assert_eq!(v.len(), 4);
    assert_eq!(v, [1, 1, 1, 1]);
    if !SM::NOOP {
        check_spare_memory(v, pattern);
    }
    v.push(2);

    v.reserve(100);
    assert!(!v.is_local());
    assert_eq!(v.capacity(), 128);
    assert_eq!(v.len(), 5);
    assert_eq!(v, [1, 1, 1, 1, 2]);
    if !SM::NOOP {
        check_spare_memory(v, pattern);
    }

    // reserve takes the MAX when next power of two overflows
    v.clear();
    for _ in 0..128 {
        v.push(7);
    }
    assert!(!v.is_local());
    assert_eq!(v.capacity(), 128);

    v.reserve(10);
    assert!(!v.is_local());
    assert_eq!(v.capacity(), 255);
    assert_eq!(v.len(), 128);
    for i in 0..v.len() {
        assert_eq!(v[i], 7);
    }
    if !SM::NOOP {
        check_spare_memory(v, pattern);
    }
}

#[test]
fn test_reserve() {
    const P: u8 = 0xCD;
    type SV = SmallVec<usize, 4, U8, Pattern<P>>;
    let mut v = SV::new();
    test_reserve_impl(&mut v, P);

    type SV2 = SmallVec<usize, 4, U8, Uninitialized>;
    let mut v = SV2::new();
    test_reserve_impl(&mut v, 0);
}

#[test]
fn test_reserve_empty() {
    type SV = SmallVec<u8, 4, U8>;
    let mut v = SV::new();
    assert_eq!(v.capacity(), 4);
    v.reserve(4);
    assert_eq!(v.capacity(), 4);
    v.reserve(8);
    assert_eq!(v.capacity(), 8);
    v.reserve(9);
    assert_eq!(v.capacity(), 16);
    v.reserve(16);
    assert_eq!(v.capacity(), 16);
    v.reserve(17);
    assert_eq!(v.capacity(), 32);
    assert!(v.is_empty());

    v.copy_from_slice(b"0123456789");
    v.copy_from_slice(b"0123456789");
    assert_eq!(v.capacity(), 32);
    v.reserve(17);
    assert_eq!(v.capacity(), 64);
}

#[test]
#[should_panic]
fn test_reserve_panics_on_length_overflow_when_local() {
    const P: u8 = 0xCD;
    type SV = SmallVec<u64, 4, U8, Pattern<P>>;

    let mut v = SV::new();
    assert!(v.is_local());
    v.reserve(256); // <-- 256 exceeds L::MAX
}

#[test]
#[should_panic]
fn test_reserve_panics_on_length_overflow_when_heap() {
    type SV = SmallVec<u64, 4, U8>;

    let mut v = SV::try_from([1, 2, 3, 4, 5]).unwrap();
    assert!(v.is_heap());
    v.reserve(256); // <-- 256 exceeds L::MAX
}

#[test]
#[should_panic]
fn test_reserve_panics_on_usize_overflow_when_local() {
    type SV = SmallVec<u64, 4>;

    let mut v = SV::new();
    assert!(v.is_local());
    v.reserve(isize::MAX as usize); // <-- isize::MAX * sizeof(u64) exceeds usize::MAX
}

#[test]
#[should_panic]
fn test_reserve_panics_on_usize_overflow_when_heap() {
    type SV = SmallVec<u64, 4, Usize>;

    let mut v = SV::with_capacity(16);
    assert!(v.is_heap());
    v.reserve(isize::MAX as usize); // <-- isize::MAX * sizeof(u64) exceeds usize::MAX
}

#[test]
#[should_panic]
fn test_reserve_zst_panics_on_length_overflow() {
    type SV = SmallVec<(), 4, U8>;

    let mut v = SV::new();
    assert_eq!(v.capacity(), u8::MAX as usize);

    v.reserve(u16::MAX as usize);
}

#[test]
fn test_zst_push() {
    gen_dropped_zst!(T);
    type SV = SmallVec<T, 4, U8>;
    let mut v = SV::new();
    for i in 0..5 {
        v.push(T::new());
        assert_eq!(v.len(), i + 1);
        assert_eq!(
            counters::<T>(),
            Counters {
                new: i + 1,
                clone: 0,
                drop: 0
            }
        );
    }
    drop(v);
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 5,
            clone: 0,
            drop: 5
        }
    );
}

#[test]
fn test_push() {
    type SV = SmallVec<usize, 4, U8>;
    let mut v = SV::new();
    for i in 0..35 {
        v.push(i);
    }
    for i in 0..35 {
        assert_eq!(v[i], i);
    }
}

#[test]
fn test_push_move_local_to_heap() {
    type SV = SmallVec<u64, 4, U8, Pattern<0xBA>>;
    let mut v = SV::new();

    for _ in 0..4 {
        v.push(1);
    }

    let old_ptr = v.as_ptr();
    assert!(v.is_local());
    v.push(2);
    assert!(v.is_heap());
    assert_ne!(old_ptr, v.as_ptr());
    assert_eq!(v, [1, 1, 1, 1, 2]);
}

#[test]
fn test_push_dropped() {
    type SV<'a> = SmallVec<Dropped<'a, 5>, 4, U8, Pattern<0xAC>>;
    let t = Track::new();
    let mut v = SV::new();
    v.push(t.alloc());
    assert!(t.dropped_indices(&[]));
    drop(v);
    assert!(t.dropped_indices(&[0]));
}

#[test]
#[should_panic]
fn test_push_panics_on_length_overflow() {
    type SV = SmallVec<usize, 4, U8>;
    let mut v = SV::new();
    for i in 0..256 {
        v.push(i);
    }
}

#[test]
fn test_zst_try_push() {
    gen_dropped_zst!(T);
    type SV = SmallVec<T, 8, U8>;
    let mut v = SV::new();

    for i in 1..11 {
        v.push(T::new());
        assert_eq!(
            counters::<T>(),
            Counters {
                new: i,
                clone: 0,
                drop: 0
            }
        );
    }

    drop(v);
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 10,
            clone: 0,
            drop: 10
        }
    );
}

#[test]
fn test_try_push() {
    type SV = SmallVec<usize, 4, U8>;
    let mut v = SV::new();
    for i in 0..35 {
        v.try_push(i).unwrap();
    }
    for i in 0..35 {
        assert_eq!(v[i], i);
    }
}

#[test]
fn test_try_push_dropped() {
    type SV<'a> = SmallVec<Dropped<'a, 5>, 4, U8, Pattern<0xAC>>;
    let t = Track::new();
    let mut v = SV::new();
    v.try_push(t.alloc()).expect("try_push failed");
    v.try_push(t.alloc()).expect("try_push failed");
    v.try_push(t.alloc()).expect("try_push failed");
    assert!(t.dropped_indices(&[]));
    drop(v);
    assert!(t.dropped_indices(&[0, 1, 2]));
}

#[test]
fn test_try_push_fails_on_length_overflow() {
    type SV = SmallVec<usize, 4, U8>;
    let mut v = SV::new();
    for i in 0..255 {
        v.try_push(i).unwrap();
    }
    assert!(matches!(
        v.try_push(256),
        Err(ReservationError::CapacityOverflow)
    ));
}

#[test]
fn test_try_push_zst_capacity_overflow() {
    type SV = SmallVec<(), 5, U8>;
    let mut v = SV::from_iter([()].iter().cloned().cycle().take(255));
    assert_eq!(v.len(), u8::MAX as usize);
    assert_eq!(v.capacity(), v.len());
    assert!(matches!(
        v.try_push(()),
        Err(ReservationError::CapacityOverflow)
    ));
}

#[test]
fn test_truncate() {
    type SV<'a> = SmallVec<Dropped<'a, 25>, 4, U8, Pattern<0xAC>>;
    let t = Track::new();
    let mut v = SV::new();
    for _ in 0..10 {
        v.push(t.alloc());
    }
    assert_eq!(v.len(), 10);
    assert!(t.dropped_indices(&[]));

    v.truncate(6);
    assert!(t.dropped_indices(&[6, 7, 8, 9]));
    assert_eq!(v.len(), 6);

    v.truncate(10);
    assert!(t.dropped_indices(&[6, 7, 8, 9]));
    assert_eq!(v.len(), 6);

    v.truncate(6);
    assert!(t.dropped_indices(&[6, 7, 8, 9]));
    assert_eq!(v.len(), 6);

    v.truncate(3);
    assert!(t.dropped_range(3..=9));
    assert_eq!(v.len(), 3);
}

fn test_try_reserve_impl<const C: usize, L, SM>(v: &mut SmallVec<usize, C, L, SM>, pattern: u8)
where
    L: LengthType,
    SM: SpareMemoryPolicy<usize>,
{
    if !SM::NOOP {
        check_spare_memory(v, pattern);
    }
    assert!(v.is_local());

    for i in 0..4 {
        v.try_reserve(1).unwrap();
        assert!(v.is_local());
        assert_eq!(v.capacity(), 4);
        assert_eq!(v.len(), i);
        if !SM::NOOP {
            check_spare_memory(v, pattern);
        }
        v.push(1);
    }

    v.try_reserve(1).unwrap();
    assert!(!v.is_local());
    assert_eq!(v.capacity(), 8);
    assert_eq!(v.len(), 4);
    assert_eq!(v, [1, 1, 1, 1]);
    if !SM::NOOP {
        check_spare_memory(v, pattern);
    }
    v.push(2);

    v.try_reserve(100).unwrap();
    assert!(!v.is_local());
    assert_eq!(v.capacity(), 128);
    assert_eq!(v.len(), 5);
    assert_eq!(v, [1, 1, 1, 1, 2]);
    if !SM::NOOP {
        check_spare_memory(v, pattern);
    }

    // reserve takes the MAX when next power of two overflows
    v.clear();
    for _ in 0..128 {
        v.push(7);
    }
    assert!(!v.is_local());
    assert_eq!(v.capacity(), 128);

    v.try_reserve(10).unwrap();
    assert!(!v.is_local());
    assert_eq!(v.capacity(), 255);
    assert_eq!(v.len(), 128);
    for i in 0..v.len() {
        assert_eq!(v[i], 7);
    }
    if !SM::NOOP {
        check_spare_memory(v, pattern);
    }
}

#[test]
fn test_try_reserve() {
    const P: u8 = 0xCD;
    type SV = SmallVec<usize, 4, U8, Pattern<P>>;
    let mut v = SV::new();
    test_try_reserve_impl(&mut v, P);

    type SV2 = SmallVec<usize, 4, U8, Uninitialized>;
    let mut v = SV2::new();
    test_try_reserve_impl(&mut v, 0);
}

#[test]
fn test_try_reserve_fails_on_length_overflow_when_heap() {
    type SV = SmallVec<u64, 4, U8>;

    let mut v = SV::try_from([1, 2, 3, 4, 5]).unwrap();
    assert!(v.is_heap());
    assert!(matches!(
        v.try_reserve(256),
        Err(ReservationError::CapacityOverflow)
    ));
}

#[test]
fn test_try_reserve_fails_on_length_overflow_when_local() {
    const P: u8 = 0xCD;
    type SV = SmallVec<u64, 4, U8, Pattern<P>>;

    let mut v = SV::new();
    assert!(v.is_local());
    assert!(matches!(
        v.try_reserve(256),
        Err(ReservationError::CapacityOverflow)
    ));
}

#[test]
fn test_try_reserve_fails_on_usize_overflow_when_heap() {
    type SV = SmallVec<u64, 4, Usize>;

    let mut v = SV::with_capacity(16);
    assert!(v.is_heap());
    assert!(matches!(
        v.try_reserve(isize::MAX as usize),
        Err(ReservationError::CapacityOverflow)
    ));
}

#[test]
fn test_try_reserve_fails_on_usize_overflow_when_local() {
    type SV = SmallVec<u64, 4>;

    let mut v = SV::new();
    assert!(v.is_local());
    assert!(matches!(
        v.try_reserve(isize::MAX as usize),
        Err(ReservationError::CapacityOverflow)
    ));
}

#[test]
fn test_try_reserve_zst_fails_on_length_overflow() {
    type SV = SmallVec<(), 4, U8>;

    let mut v = SV::new();
    assert_eq!(v.capacity(), u8::MAX as usize);

    assert!(matches!(
        v.try_reserve(u16::MAX as usize),
        Err(ReservationError::CapacityOverflow)
    ));
}

fn test_try_reserve_exact_impl<const C: usize, L, SM>(
    v: &mut SmallVec<usize, C, L, SM>,
    pattern: u8,
) where
    L: LengthType,
    SM: SpareMemoryPolicy<usize>,
{
    if !SM::NOOP {
        check_spare_memory(v, pattern);
    }
    assert!(v.is_local());

    for i in 0..4 {
        v.try_reserve_exact(1).unwrap();
        assert!(v.is_local());
        assert_eq!(v.capacity(), 4);
        assert_eq!(v.len(), i);
        if !SM::NOOP {
            check_spare_memory(v, pattern);
        }
        v.push(1);
    }

    v.try_reserve_exact(1).unwrap();
    assert!(!v.is_local());
    assert_eq!(v.capacity(), 5);
    assert_eq!(v.len(), 4);
    assert_eq!(v, [1, 1, 1, 1]);
    if !SM::NOOP {
        check_spare_memory(v, pattern);
    }
    v.push(2);

    v.try_reserve_exact(100).unwrap();
    assert!(!v.is_local());
    assert_eq!(v.capacity(), 105);
    assert_eq!(v.len(), 5);
    assert_eq!(v, [1, 1, 1, 1, 2]);
    if !SM::NOOP {
        check_spare_memory(v, pattern);
    }

    // reserve takes the MAX when next power of two overflows
    v.clear();
    for _ in 0..128 {
        v.push(7);
    }
    assert!(!v.is_local());
    assert_eq!(v.capacity(), 128);

    v.try_reserve_exact(10).unwrap();
    assert!(!v.is_local());
    assert_eq!(v.capacity(), 138);
    assert_eq!(v.len(), 128);
    for i in 0..v.len() {
        assert_eq!(v[i], 7);
    }
    if !SM::NOOP {
        check_spare_memory(v, pattern);
    }
}

#[test]
fn test_try_reserve_exact() {
    const P: u8 = 0xCD;
    type SV = SmallVec<usize, 4, U8, Pattern<P>>;
    let mut v = SV::new();
    test_try_reserve_exact_impl(&mut v, P);

    type SV2 = SmallVec<usize, 4, U8, Uninitialized>;
    let mut v = SV2::new();
    test_try_reserve_exact_impl(&mut v, 0);
}

#[test]
fn test_try_from_iter() {
    type SV = SmallVec<usize, 4, U8>;
    let v = SV::try_from_iter(0..3).unwrap();
    assert_eq!(v, [0, 1, 2]);

    let v = SV::try_from_iter(0..5).unwrap();
    assert_eq!(v, [0, 1, 2, 3, 4]);
}

#[test]
fn test_try_from_iter_fails_on_iterator_size_hint() {
    let (min, max) = (0..256).size_hint();
    assert_eq!(min, 256);
    assert_eq!(max, Some(256));

    type SV = SmallVec<usize, 4, U8>;
    assert!(matches!(
        SV::try_from_iter(0..256),
        Err(ReservationError::CapacityOverflow)
    ));
}

#[test]
fn test_try_from_iter_fails_without_valid_hint() {
    struct I {
        max: usize,
        cnt: usize,
    }

    impl Iterator for I {
        type Item = usize;

        fn next(&mut self) -> Option<Self::Item> {
            if self.cnt < self.max {
                self.cnt += 1;
                Some(self.cnt)
            } else {
                None
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (0, None)
        }
    }

    type SV = SmallVec<usize, 4, U8>;
    assert!(matches!(
        SV::try_from_iter(I { max: 256, cnt: 0 }),
        Err(ReservationError::CapacityOverflow)
    ));
}

#[test]
fn test_try_from_iter_drops_on_failure_without_valid_hint() {
    type Item<'a> = Dropped<'a, 512>;
    type SV<'a> = SmallVec<Item<'a>, 64, U8>;
    let t = Track::new();

    struct I<'a, const C: usize> {
        track: &'a Track<C>,
        max: usize,
        cnt: usize,
    }

    impl<'a, const C: usize> Iterator for I<'a, C> {
        type Item = Dropped<'a, C>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.cnt < self.max {
                self.cnt += 1;
                Some(self.track.alloc())
            } else {
                None
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (0, None)
        }
    }

    assert!(t.dropped_range(0..0)); // empty range
    assert!(matches!(
        SV::try_from_iter(I {
            track: &t,
            max: 320,
            cnt: 0
        }),
        Err(ReservationError::CapacityOverflow)
    ));
    assert!(t.dropped_range(0..=255)); // U8 overflows after 255 items
}

#[test]
fn test_zst_pop_local() {
    gen_dropped_zst!(T);
    type SV = SmallVec<T, 15>;
    let mut v = SV::new();

    fn check(v: &mut SV, cap: usize) {
        for _ in 0..cap {
            v.push(T::new());
        }
        assert_eq!(v.len(), cap);
        assert_eq!(v.capacity(), usize::MAX);
        assert_eq!(
            counters::<T>(),
            Counters {
                new: cap,
                clone: 0,
                drop: 0
            }
        );

        for i in 0..cap {
            assert!(v.pop().is_some());
            assert_eq!(v.len(), cap - i - 1);
            assert_eq!(
                counters::<T>(),
                Counters {
                    new: cap,
                    clone: 0,
                    drop: i + 1,
                }
            );
        }
        assert_eq!(v.pop(), None);
    }

    check(&mut v, 15);
}

#[test]
fn test_zst_pop_heap() {
    gen_dropped_zst!(T);
    type SV = SmallVec<T, 15>;
    let mut v = SV::new();

    fn check(v: &mut SV, cap: usize) {
        for _ in 0..cap {
            v.push(T::new());
        }
        assert_eq!(v.len(), cap);
        assert_eq!(v.capacity(), usize::MAX);
        assert_eq!(
            counters::<T>(),
            Counters {
                new: cap,
                clone: 0,
                drop: 0
            }
        );

        for i in 0..cap {
            assert!(v.pop().is_some());
            assert_eq!(v.len(), cap - i - 1);
            assert_eq!(
                counters::<T>(),
                Counters {
                    new: cap,
                    clone: 0,
                    drop: i + 1,
                }
            );
        }
        assert_eq!(v.pop(), None);
    }

    check(&mut v, 30);
}

#[test]
fn test_pop() {
    const P: u8 = 0xEF;
    type Item<'a> = Dropped<'a, 512>;
    type SV<'a> = SmallVec<Item<'a>, 8, U8, Pattern<P>>;
    let t = Track::new();

    let mut v = SV::from_iter(t.take(8));
    assert!(v.is_local());
    assert_eq!(v.len(), 8);
    assert_eq!(v.capacity(), 8);

    for i in 0..8 {
        assert!(v.pop().is_some());
        assert_eq!(v.len(), 8 - i - 1);
        assert_eq!(v.capacity(), 8);
        check_spare_memory(&v, P);
    }
    assert!(v.pop().is_none());
    assert!(v.is_empty());
    t.dropped_range(0..8);

    let mut v = SV::from_iter(t.take(16));
    assert!(v.is_heap());
    assert_eq!(v.len(), 16);
    assert_eq!(v.capacity(), 16);

    for i in 0..16 {
        assert!(v.pop().is_some());
        assert_eq!(v.len(), 16 - i - 1);
        assert_eq!(v.capacity(), 16);
        check_spare_memory(&v, P);
    }
    assert!(v.pop().is_none());
    assert!(v.is_empty());
    t.dropped_range(0..24);
}

#[test]
fn test_zst_try_insert() {
    gen_dropped_zst!(T);
    type SV = SmallVec<T, 2, U8>;
    let mut v = SV::new();
    v.try_insert(0, T::new()).unwrap();
    v.try_insert(0, T::new()).unwrap();
    assert_eq!(v.capacity(), u8::MAX as usize);
    assert_eq!(v.len(), 2);
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 2,
            clone: 0,
            drop: 0,
        }
    );

    for _ in 2..255 {
        v.try_insert(0, T::new()).unwrap();
    }
    assert_eq!(v.len(), u8::MAX as usize);
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 255,
            clone: 0,
            drop: 0,
        }
    );

    assert!(matches!(
        v.try_insert(312, T::new()),
        Err(InsertError::InvalidIndex)
    ));

    assert!(matches!(
        v.try_insert(0, T::new()),
        Err(InsertError::ReservationError(
            ReservationError::CapacityOverflow
        ))
    ));
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 257,
            clone: 0,
            drop: 2,
        }
    );

    drop(v);
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 257,
            clone: 0,
            drop: 257,
        }
    );
}

#[test]
fn test_try_insert_local() {
    let mut v = small_vec![16; u64; 1, 2, 4];
    assert!(v.is_local());
    v.try_insert(2, 3).unwrap();
    assert_eq!(v, [1, 2, 3, 4]);

    assert!(matches!(v.try_insert(5, 5), Err(InsertError::InvalidIndex)));
}

#[test]
fn test_try_insert_heap() {
    let mut v = small_vec![1; u64; 1, 2, 4];
    assert!(v.is_heap());
    v.try_insert(2, 3).unwrap();
    assert_eq!(v, [1, 2, 3, 4]);

    assert!(matches!(v.try_insert(5, 5), Err(InsertError::InvalidIndex)));
}

#[test]
fn test_try_insert_reservation_error_heap() {
    type SV = SmallVec<u64, 8, U8>;
    let mut v = SV::from_iter(0..255);

    assert!(matches!(
        v.try_insert(5, 5),
        Err(InsertError::ReservationError(
            re
        )) if re == ReservationError::CapacityOverflow
    ));
}

#[test]
fn test_try_insert_local_to_heap() {
    let mut v = small_vec![3; u64; 1, 2, 4];
    assert!(v.is_local());

    assert!(matches!(v.try_insert(5, 5), Err(InsertError::InvalidIndex)));

    v.try_insert(2, 3).unwrap();
    assert!(v.is_heap());
    assert_eq!(v, [1, 2, 3, 4]);
}

#[test]
fn test_try_insert_reservation_error_local_to_heap() {
    type SV = SmallVec<u64, 255, U8>;
    let mut v = SV::from_iter(0..255);

    assert!(matches!(
        v.try_insert(5, 5),
        Err(InsertError::ReservationError(
            re
        )) if re == ReservationError::CapacityOverflow
    ));
}

#[test]
fn test_try_insert_dropped() {
    const P: u8 = 0xEF;
    type Item<'a> = Dropped<'a, 512>;
    type SV<'a> = SmallVec<Item<'a>, 8, U8, Pattern<P>>;
    let t = Track::new();

    let mut v = SV::from_iter(t.take(8));
    check_spare_memory(&v, P);
    assert_eq!(t.n_allocated(), 8);
    assert!(t.dropped_range(0..0));

    v.try_insert(0, t.alloc()).unwrap();
    check_spare_memory(&v, P);
    assert_eq!(v.len(), 9);
    assert_eq!(t.n_allocated(), 9);
    assert!(t.dropped_range(0..0));

    drop(v);
    assert!(t.dropped_range(0..9));
}

#[test]
fn test_insert() {
    let mut v = small_vec![5; u64; 0, 0, 0];
    v.insert(0, 1);
    assert_eq!(v, [1, 0, 0, 0]);
    v.insert(4, 1);
    assert_eq!(v, [1, 0, 0, 0, 1]);
    v.insert(5, 3);
    assert_eq!(v, [1, 0, 0, 0, 1, 3]);
    v.insert(6, 4);
    assert_eq!(v, [1, 0, 0, 0, 1, 3, 4]);
}

#[test]
#[should_panic]
fn test_insert_panics_on_invalid_index_local() {
    let mut v = small_vec![6; u64; 0, 0, 0];
    v.insert(5, 1);
}

#[test]
#[should_panic]
fn test_insert_panics_on_invalid_index_heap() {
    let mut v = small_vec![1; u64; 0, 0, 0];
    v.insert(5, 1);
}

#[test]
#[should_panic]
fn test_insert_panics_on_invalid_index_local_to_heap() {
    let mut v = small_vec![3; u64; 0, 0, 0];
    v.insert(5, 1);
}

#[test]
#[should_panic]
fn test_insert_panics_on_reservation_error_local_to_heap() {
    type SV = SmallVec<u64, 255, U8>;
    let mut v = SV::from_iter(0..255);
    v.insert(0, 1);
}

#[test]
#[should_panic]
fn test_insert_panics_on_reservation_error_heap() {
    type SV = SmallVec<u64, 2, U8>;
    let mut v = SV::from_iter(0..255);
    v.insert(0, 1);
}

#[test]
#[allow(clippy::unit_cmp)]
fn test_zst_remove() {
    type SV = SmallVec<(), 8, U8>;
    let mut v = SV::new();
    for _ in 0..16 {
        v.push(());
    }
    assert_eq!(v.remove(0), ());
}

#[test]
#[should_panic]
fn test_zst_remove_panics() {
    type SV = SmallVec<(), 8, U8>;
    let mut v = SV::new();
    for _ in 0..16 {
        v.push(());
    }
    v.remove(32);
}

#[test]
fn test_remove_local() {
    const P: u8 = 0xEF;
    type Item<'a> = Dropped<'a, 512>;
    type SV<'a> = SmallVec<Item<'a>, 8, U8, Pattern<P>>;
    let t = Track::new();

    let mut v = SV::from_iter(t.take(8));
    assert!(v.is_local());
    assert_eq!(t.n_allocated(), 8);
    assert!(t.dropped_range(0..0));
    check_spare_memory(&v, P);

    v.remove(2);
    assert!(t.dropped_indices(&[2]));
    check_spare_memory(&v, P);

    v.remove(2);
    assert!(t.dropped_indices(&[2, 3]));
    check_spare_memory(&v, P);

    drop(v);
    assert!(t.dropped_range(0..8));
}

#[test]
#[should_panic]
fn test_remove_local_panics() {
    let mut v = small_vec![8; u16; 1, 2, 3, 4, 5];
    assert!(v.is_local());
    v.remove(10);
}

#[test]
fn test_remove_heap() {
    const P: u8 = 0xEF;
    type Item<'a> = Dropped<'a, 512>;
    type SV<'a> = SmallVec<Item<'a>, 1, U8, Pattern<P>>;
    let t = Track::new();

    let mut v = SV::from_iter(t.take(8));
    assert!(v.is_heap());
    assert_eq!(t.n_allocated(), 8);
    assert!(t.dropped_range(0..0));
    check_spare_memory(&v, P);

    v.remove(4);
    assert!(t.dropped_indices(&[4]));
    check_spare_memory(&v, P);

    v.remove(4);
    assert!(t.dropped_indices(&[4, 5]));
    check_spare_memory(&v, P);

    drop(v);
    assert!(t.dropped_range(0..8));
}

#[test]
#[should_panic]
fn test_remove_heap_panics() {
    let mut v = small_vec![2; u16; 1, 2, 3, 4, 5];
    assert!(v.is_heap());
    v.remove(10);
}

#[test]
fn test_zst_try_remove() {
    gen_dropped_zst!(T);
    type SV = SmallVec<T, 8, U8>;
    let mut v = SV::new();
    for _ in 0..8 {
        v.push(T::new());
    }
    assert_eq!(v.len(), 8);
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 8,
            clone: 0,
            drop: 0
        }
    );
    assert!(v.try_remove(2).is_some());
    assert_eq!(v.len(), 7);
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 8,
            clone: 0,
            drop: 1,
        }
    );
    assert_eq!(v.try_remove(10), None);
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 8,
            clone: 0,
            drop: 1,
        }
    );

    drop(v);
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 8,
            clone: 0,
            drop: 8,
        }
    );
}

#[test]
fn test_try_remove_local() {
    const P: u8 = 0xEF;
    type Item<'a> = Dropped<'a, 512>;
    type SV<'a> = SmallVec<Item<'a>, 8, U8, Pattern<P>>;
    let t = Track::new();

    let mut v = SV::from_iter(t.take(8));
    assert!(v.is_local());
    assert_eq!(t.n_allocated(), 8);
    assert!(t.dropped_range(0..0));
    check_spare_memory(&v, P);

    assert!(v.try_remove(2).is_some());
    assert!(t.dropped_indices(&[2]));
    check_spare_memory(&v, P);

    assert!(v.try_remove(10).is_none());

    drop(v);
    assert!(t.dropped_range(0..8));
}

#[test]
fn test_try_remove_heap() {
    const P: u8 = 0xEF;
    type Item<'a> = Dropped<'a, 512>;
    type SV<'a> = SmallVec<Item<'a>, 2, U8, Pattern<P>>;
    let t = Track::new();

    let mut v = SV::from_iter(t.take(8));
    assert!(v.is_heap());
    assert_eq!(t.n_allocated(), 8);
    assert!(t.dropped_range(0..0));
    check_spare_memory(&v, P);

    assert!(v.try_remove(2).is_some());
    assert!(t.dropped_indices(&[2]));
    check_spare_memory(&v, P);

    assert!(v.try_remove(10).is_none());

    drop(v);
    assert!(t.dropped_range(0..8));
}

#[test]
fn test_try_remove_empty_sv() {
    let mut v = small_vec![8; u64];
    assert!(v.try_remove(0).is_none());
}

#[test]
#[allow(clippy::unit_cmp)]
fn test_zst_swap_remove() {
    type SV = SmallVec<(), 8, U8>;
    let mut v = SV::new();
    for _ in 0..8 {
        v.push(());
    }
    assert_eq!(v.swap_remove(2), ());
}

#[test]
#[should_panic]
fn test_zst_swap_remove_panics() {
    type SV = SmallVec<(), 8, U8>;
    let mut v = SV::new();
    v.swap_remove(0);
}

#[test]
fn test_swap_remove_local() {
    const P: u8 = 0xAF;
    type Item<'a> = Dropped<'a, 512>;
    type SV<'a> = SmallVec<Item<'a>, 8, U8, Pattern<P>>;
    let t = Track::new();

    let mut v = SV::from_iter(t.take(8));
    assert!(v.is_local());

    assert_eq!(v.swap_remove(1).idx(), 1);
    assert!(t.dropped_indices(&[1]));

    assert_eq!(v.swap_remove(1).idx(), 7);
    assert!(t.dropped_indices(&[1, 7]));
}

#[test]
#[should_panic]
fn test_swap_remove_local_panics() {
    let mut v = small_vec![8; u64];
    v.extend(0..8);
    assert!(v.is_local());
    v.swap_remove(10);
}

#[test]
fn test_swap_remove_heap() {
    const P: u8 = 0xAF;
    type Item<'a> = Dropped<'a, 512>;
    type SV<'a> = SmallVec<Item<'a>, 1, U8, Pattern<P>>;
    let t = Track::new();

    let mut v = SV::from_iter(t.take(8));
    assert!(v.is_heap());

    assert_eq!(v.swap_remove(1).idx(), 1);
    assert!(t.dropped_indices(&[1]));

    assert_eq!(v.swap_remove(1).idx(), 7);
    assert!(t.dropped_indices(&[1, 7]));
}

#[test]
#[should_panic]
fn test_swap_remove_heap_panics() {
    let mut v = small_vec![2; u64];
    v.extend(0..8);
    assert!(v.is_heap());
    v.swap_remove(10);
}

#[test]
fn test_zst_try_swap_remove() {
    gen_dropped_zst!(T);
    type SV = SmallVec<T, 4, U8>;
    let mut v = SV::new();
    for _ in 0..16 {
        v.push(T::new());
    }
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 16,
            clone: 0,
            drop: 0
        }
    );
    assert!(v.try_swap_remove(0).is_some());
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 16,
            clone: 0,
            drop: 1
        }
    );
    assert_eq!(v.try_swap_remove(20), None);
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 16,
            clone: 0,
            drop: 1
        }
    );
    drop(v);
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 16,
            clone: 0,
            drop: 16,
        }
    );
}

#[test]
fn test_try_swap_remove_local() {
    const P: u8 = 0xAF;
    type SV = SmallVec<u64, 8, U8, Pattern<P>>;
    let mut v = SV::from_iter(0..8);
    assert!(v.is_local());

    assert_eq!(v.try_swap_remove(2), Some(2));
    check_spare_memory(&v, P);
    assert_eq!(v.try_swap_remove(2), Some(7));
    check_spare_memory(&v, P);
    assert_eq!(v.try_swap_remove(99), None);
    assert_eq!(v, [0, 1, 6, 3, 4, 5]);
    check_spare_memory(&v, P);
}

#[test]
fn test_try_swap_remove_heap() {
    const P: u8 = 0xAF;
    type SV = SmallVec<u64, 1, U8, Pattern<P>>;
    let mut v = SV::from_iter(0..8);
    assert!(v.is_heap());

    assert_eq!(v.try_swap_remove(2), Some(2));
    check_spare_memory(&v, P);
    assert_eq!(v.try_swap_remove(2), Some(7));
    check_spare_memory(&v, P);
    assert_eq!(v.try_swap_remove(99), None);
    assert_eq!(v, [0, 1, 6, 3, 4, 5]);
    check_spare_memory(&v, P);
}

#[test]
fn test_try_swap_remove_empty_vector() {
    let mut v = small_vec![8; u64];
    assert!(v.try_swap_remove(0).is_none());
}

#[test]
fn test_retain_dropped() {
    const P: u8 = 0xAF;
    type Item<'a> = Dropped<'a, 512>;
    type SV<'a> = SmallVec<Item<'a>, 1, U8, Pattern<P>>;
    let t = Track::new();

    let mut v = SV::from_iter(t.take(8));
    assert_eq!(v.len(), 8);
    assert!(t.dropped_range(0..0));
    check_spare_memory(&v, P);

    v.retain(|e| e.idx() != 3);
    assert_eq!(v.len(), 7);
    assert!(t.dropped_range(3..=3));
    check_spare_memory(&v, P);

    v.retain(|e| e.idx() % 2 == 0);
    assert_eq!(v.len(), 4);
    assert!(t.dropped_indices(&[1, 3, 5, 7]));
    check_spare_memory(&v, P);

    drop(v);
    assert!(t.dropped_range(0..8));
}

#[test]
fn test_retain() {
    let mut v = small_vec![8; u64; 0, 1, 2, 3, 4, 5, 6, 7];
    assert_eq!(v, [0, 1, 2, 3, 4, 5, 6, 7]);
    v.retain(|e| *e != 4);
    assert_eq!(v, [0, 1, 2, 3, 5, 6, 7]);
    v.retain(|e| *e % 3 != 0);
    assert_eq!(v, [1, 2, 5, 7]);
}

#[test]
fn test_retain_mut() {
    let mut v = small_vec![8; u64; 0, 1, 2, 3, 4, 5, 6, 7];
    assert_eq!(v, [0, 1, 2, 3, 4, 5, 6, 7]);
    v.retain_mut(|e| {
        *e *= 3;
        *e % 2 == 0
    });
    assert_eq!(v, [0, 6, 12, 18]);
}

#[test]
fn test_zst_try_resize_with() {
    gen_dropped_zst!(T);
    type SV = SmallVec<T, 8, U8>;
    let mut v = SV::new();
    assert_eq!(v.len(), 0);
    assert!(v.try_resize_with(255, T::new).is_ok());
    assert_eq!(v.len(), 255);
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 255,
            clone: 0,
            drop: 0
        }
    );

    assert!(matches!(
        v.try_resize_with(256, T::new),
        Err(ReservationError::CapacityOverflow)
    ));

    assert_eq!(
        counters::<T>(),
        Counters {
            new: 255,
            clone: 0,
            drop: 0
        }
    );

    drop(v);
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 255,
            clone: 0,
            drop: 255,
        }
    );
}

#[test]
fn test_try_resize_with_local() {
    const P: u8 = 0xDD;
    type SV = SmallVec<u64, 8, U8, Pattern<P>>;
    let mut v = SV::new();
    let mut cnt = 0;

    assert!(v
        .try_resize_with(5, || {
            cnt += 1;
            cnt
        })
        .is_ok());
    assert_eq!(v, [1, 2, 3, 4, 5]);
    check_spare_memory(&v, P);

    assert!(v.try_resize_with(3, || 0).is_ok());
    assert_eq!(v, [1, 2, 3]);
    check_spare_memory(&v, P);

    assert!(v.try_resize_with(8, || 0).is_ok());
    assert_eq!(v, [1, 2, 3, 0, 0, 0, 0, 0]);
    check_spare_memory(&v, P);

    assert!(v.try_resize_with(8, || 1).is_ok());
    assert_eq!(v, [1, 2, 3, 0, 0, 0, 0, 0]);
    check_spare_memory(&v, P);

    assert!(v.try_resize_with(10, || 7).is_ok());
    assert_eq!(v, [1, 2, 3, 0, 0, 0, 0, 0, 7, 7]);
    check_spare_memory(&v, P);

    assert!(v.try_resize_with(10, || 1).is_ok());
    assert_eq!(v, [1, 2, 3, 0, 0, 0, 0, 0, 7, 7]);

    assert!(matches!(
        v.try_resize_with(256, || 1),
        Err(ReservationError::CapacityOverflow)
    ));
}

#[test]
fn test_try_resize_with_heap() {
    const P: u8 = 0xEE;
    type SV = SmallVec<u64, 0, U8, Pattern<P>>;
    let mut v = SV::new();
    let mut cnt = 0;

    assert!(v
        .try_resize_with(5, || {
            cnt += 1;
            cnt
        })
        .is_ok());
    assert_eq!(v, [1, 2, 3, 4, 5]);
    check_spare_memory(&v, P);

    assert!(v.try_resize_with(3, || 0).is_ok());
    assert_eq!(v, [1, 2, 3]);
    check_spare_memory(&v, P);

    assert!(v.try_resize_with(8, || 0).is_ok());
    assert_eq!(v, [1, 2, 3, 0, 0, 0, 0, 0]);
    check_spare_memory(&v, P);

    assert!(v.try_resize_with(8, || 1).is_ok());
    assert_eq!(v, [1, 2, 3, 0, 0, 0, 0, 0]);
    check_spare_memory(&v, P);

    assert!(v.try_resize_with(10, || 7).is_ok());
    assert_eq!(v, [1, 2, 3, 0, 0, 0, 0, 0, 7, 7]);
    check_spare_memory(&v, P);

    assert!(v.try_resize_with(10, || 1).is_ok());
    assert_eq!(v, [1, 2, 3, 0, 0, 0, 0, 0, 7, 7]);

    assert!(matches!(
        v.try_resize_with(256, || 3),
        Err(ReservationError::CapacityOverflow)
    ));
}

#[test]
fn test_zst_resize_with() {
    type SV = SmallVec<(), 8, U8>;
    let mut v = SV::new();
    v.resize_with(3, || ());
    assert_eq!(v.len(), 3);
    v.resize_with(10, || ());
    assert_eq!(v.len(), 10);
    v.resize_with(3, || ());
    assert_eq!(v.len(), 3);
}

#[test]
#[should_panic]
fn test_zst_resize_with_panics() {
    type SV = SmallVec<(), 8, U8>;
    let mut v = SV::new();
    v.resize_with(255, || ());
    v.resize_with(256, || ());
}

#[test]
fn test_resize_with_local() {
    const P: u8 = 0xAA;
    type SV = SmallVec<u64, 8, U8, Pattern<P>>;
    let mut v = SV::new();
    let mut cnt = 0;
    assert_eq!(v, []);
    check_spare_memory(&v, P);

    v.resize_with(4, || {
        cnt += 1;
        cnt
    });
    assert_eq!(v, [1, 2, 3, 4]);
    check_spare_memory(&v, P);

    v.resize_with(8, || {
        cnt += 1;
        cnt
    });
    assert_eq!(v, [1, 2, 3, 4, 5, 6, 7, 8]);
    check_spare_memory(&v, P);

    v.resize_with(8, || {
        cnt += 1;
        cnt
    });
    assert_eq!(v, [1, 2, 3, 4, 5, 6, 7, 8]);
    check_spare_memory(&v, P);

    v.resize_with(2, || 0);
    assert_eq!(v, [1, 2]);
    check_spare_memory(&v, P);

    v.resize_with(10, || {
        cnt += 1;
        cnt
    });
    assert_eq!(v, [1, 2, 9, 10, 11, 12, 13, 14, 15, 16]);
    check_spare_memory(&v, P);
}

#[test]
#[should_panic]
fn test_resize_with_local_panics() {
    type SV = SmallVec<u64, 8, U8>;
    let mut v = SV::from_iter(0..5);
    v.resize_with(256, || 0);
}

#[test]
fn test_resize_with_heap() {
    const P: u8 = 0xAA;
    type SV = SmallVec<u64, 1, U8, Pattern<P>>;
    let mut cnt = 700;

    let mut v = SV::from_iter(0..2);
    assert_eq!(v, [0, 1]);
    check_spare_memory(&v, P);

    v.resize_with(1, || 0);
    assert_eq!(v, [0]);
    check_spare_memory(&v, P);

    v.resize_with(2, || {
        cnt += 1;
        cnt
    });
    assert_eq!(v, [0, 701]);
    check_spare_memory(&v, P);

    v.resize_with(5, || {
        cnt += 1;
        cnt
    });
    assert_eq!(v, [0, 701, 702, 703, 704]);
    check_spare_memory(&v, P);

    v.resize_with(5, || 0);
    assert_eq!(v, [0, 701, 702, 703, 704]);
    check_spare_memory(&v, P);
}

#[test]
fn test_resize_with_dropped() {
    const P: u8 = 0xFF;
    type Item<'a> = Dropped<'a, 512>;
    type SV<'a> = SmallVec<Item<'a>, 4, U8, Pattern<P>>;
    let t = Track::new();

    let mut v = SV::new();
    v.resize_with(3, || t.alloc());
    assert_eq!(v.len(), 3);
    assert!(t.dropped_range(0..0));

    v.resize_with(1, || t.alloc());
    assert_eq!(v.len(), 1);
    assert!(t.dropped_range(1..3));

    v.resize_with(5, || t.alloc());
    assert_eq!(v.len(), 5);

    drop(v);
    assert!(t.dropped_range(0..7));
}

#[test]
fn test_try_resize_with_dropped() {
    const P: u8 = 0xFF;
    type Item<'a> = Dropped<'a, 512>;
    type SV<'a> = SmallVec<Item<'a>, 4, U8, Pattern<P>>;
    let t = Track::new();

    let mut v = SV::new();
    v.try_resize_with(3, || t.alloc()).unwrap();
    assert_eq!(v.len(), 3);
    assert!(t.dropped_range(0..0));

    v.try_resize_with(1, || t.alloc()).unwrap();
    assert_eq!(v.len(), 1);
    assert!(t.dropped_range(1..3));

    v.try_resize_with(5, || t.alloc()).unwrap();
    assert_eq!(v.len(), 5);

    drop(v);
    assert!(t.dropped_range(0..7));
}

#[test]
#[should_panic]
fn test_resize_with_heap_panics() {
    type SV = SmallVec<u64, 1, U8>;
    let mut v = SV::from_iter(0..5);
    v.resize_with(256, || 0);
}

#[test]
fn test_zst_try_resize() {
    gen_dropped_zst!(T);
    type SV = SmallVec<T, 8, U8>;
    let mut v = SV::new();
    assert!(v.try_resize(4, T::new()).is_ok());
    assert_eq!(v.len(), 4);
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 1,
            clone: 3,
            drop: 0
        }
    );

    assert!(v.try_resize(8, T::new()).is_ok());
    assert_eq!(v.len(), 8);
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 2,
            clone: 6,
            drop: 0
        }
    );

    assert!(v.try_resize(3, T::new()).is_ok());
    assert_eq!(v.len(), 3);
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 3,
            clone: 6,
            drop: 6,
        }
    );

    assert!(v.try_resize(255, T::new()).is_ok());
    assert_eq!(v.len(), 255);
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 4,
            clone: 257,
            drop: 6,
        }
    );

    assert!(matches!(
        v.try_resize(256, T::new()),
        Err(ReservationError::CapacityOverflow)
    ));

    drop(v);
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 5,
            clone: 257,
            drop: 262,
        }
    );
}

#[test]
fn test_try_resize_local() {
    const P: u8 = 0xBB;
    type SV = SmallVec<u64, 8, U8, Pattern<P>>;
    let mut v = SV::new();
    assert_eq!(v.len(), 0);
    check_spare_memory(&v, P);

    assert!(v.try_resize(4, 7).is_ok());
    assert_eq!(v, [7, 7, 7, 7]);
    check_spare_memory(&v, P);

    assert!(v.try_resize(8, 1).is_ok());
    assert_eq!(v, [7, 7, 7, 7, 1, 1, 1, 1]);
    check_spare_memory(&v, P);

    assert!(v.try_resize(5, 0).is_ok());
    assert_eq!(v, [7, 7, 7, 7, 1]);
    check_spare_memory(&v, P);

    assert!(v.try_resize(5, 100).is_ok());
    assert_eq!(v, [7, 7, 7, 7, 1]);
    check_spare_memory(&v, P);

    assert!(matches!(
        v.try_resize(256, 17),
        Err(ReservationError::CapacityOverflow)
    ));

    assert!(v.try_resize(255, 0).is_ok());
    assert_eq!(v.len(), 255);
    check_spare_memory(&v, P);
}

#[test]
fn test_try_resize_heap() {
    const P: u8 = 0xAA;
    type SV = SmallVec<u64, 2, U8, Pattern<P>>;
    let mut v = SV::from_iter(0..3);
    assert_eq!(v, [0, 1, 2]);
    check_spare_memory(&v, P);

    v.try_resize(1, 0).unwrap();
    assert_eq!(v, [0]);

    v.try_resize(2, 7).unwrap();
    assert_eq!(v, [0, 7]);

    v.try_resize(2, 5).unwrap();
    assert_eq!(v, [0, 7]);

    v.try_resize(5, 9).unwrap();
    assert_eq!(v, [0, 7, 9, 9, 9]);

    assert!(matches!(
        v.try_resize(256, 17),
        Err(ReservationError::CapacityOverflow)
    ));
}

#[test]
fn test_zst_resize() {
    type SV = SmallVec<(), 8, U8>;
    let mut v = SV::new();
    assert_eq!(v.len(), 0);
    v.resize(3, ());
    assert_eq!(v.len(), 3);
    v.resize(8, ());
    assert_eq!(v.len(), 8);
    v.resize(0, ());
    assert_eq!(v.len(), 0);
}

#[test]
#[should_panic]
fn test_zst_resize_panics() {
    type SV = SmallVec<(), 8, U8>;
    let mut v = SV::new();
    v.resize(256, ());
}

#[test]
fn test_resize_local() {
    const P: u8 = 0xFE;
    type SV = SmallVec<u64, 8, U8, Pattern<P>>;
    let mut v = SV::new();
    v.resize(2, 0);
    assert_eq!(v, [0, 0]);
    check_spare_memory(&v, P);

    v.resize(8, 1);
    assert_eq!(v, [0, 0, 1, 1, 1, 1, 1, 1]);
    check_spare_memory(&v, P);

    v.resize(1, 3);
    assert_eq!(v, [0]);
    check_spare_memory(&v, P);

    v.resize(1, 5);
    assert_eq!(v, [0]);
    check_spare_memory(&v, P);

    v.resize(10, 5);
    assert_eq!(v, [0, 5, 5, 5, 5, 5, 5, 5, 5, 5]);
    check_spare_memory(&v, P);
}

#[test]
#[should_panic]
fn test_resize_local_panics() {
    type SV = SmallVec<u64, 8, U8>;
    let mut v = SV::from_iter(1..3);
    v.resize(8, 0);
    v.resize(256, 1);
}

#[test]
fn test_resize_heap() {
    const P: u8 = 0xFE;
    type SV = SmallVec<u64, 2, U8, Pattern<P>>;
    let mut v = SV::from_iter(1..4);
    assert_eq!(v, [1, 2, 3]);
    check_spare_memory(&v, P);

    v.resize(5, 0);
    assert_eq!(v, [1, 2, 3, 0, 0]);
    check_spare_memory(&v, P);

    v.resize(2, 1);
    assert_eq!(v, [1, 2]);
    check_spare_memory(&v, P);

    v.resize(2, 7);
    assert_eq!(v, [1, 2]);
    check_spare_memory(&v, P);

    v.resize(10, 5);
    assert_eq!(v, [1, 2, 5, 5, 5, 5, 5, 5, 5, 5]);
    check_spare_memory(&v, P);
}

#[test]
#[should_panic]
fn test_drain_end_out_of_bounds() {
    let mut v = small_vec![16; u64];
    v.extend(0..16);
    v.drain(0..17);
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

    let mut v = small_vec![3; u64; 1, 2, 3];
    v.drain(r);
}

#[test]
#[should_panic]
fn test_drain_end_overflow() {
    let start = 0;
    let end = usize::MAX;

    let r = CustomRange {
        start: Bound::Included(&start),
        end: Bound::Included(&end),
    };

    let mut v = small_vec![3; u64; 1, 2, 3];
    v.drain(r);
}

#[test]
#[should_panic]
fn test_drain_start_out_of_bounds() {
    let start = 2;
    let end = 1usize;

    let r = CustomRange {
        start: Bound::Excluded(&start),
        end: Bound::Included(&end),
    };

    let mut v = small_vec![3; u64; 1, 2, 3];
    v.drain(r);
}

#[test]
fn test_drain_empty_range() {
    let mut v = small_vec![8; u64; 1, 2, 3];
    let mut d = v.drain(1..1);
    assert_eq!(d.next(), None);
    drop(d);
    assert_eq!(v, [1, 2, 3]);
}

#[test]
fn test_drain_zst() {
    gen_dropped_zst!(T);
    type SV = SmallVec<T, 8, U8>;
    let mut v = SV::new();
    v.resize_with(8, T::new);
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 8,
            clone: 0,
            drop: 0
        }
    );
    let c = T {};
    for e in v.drain(1..3) {
        assert_eq!(e, c);
    }
    assert_eq!(v.len(), 6);
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 8,
            clone: 0,
            drop: 2,
        }
    );

    v.drain(..);
    assert_eq!(v.len(), 0);
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 8,
            clone: 0,
            drop: 8
        }
    );

    drop(v);
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 8,
            clone: 0,
            drop: 8
        }
    );
}

#[test]
fn test_drain_all_local() {
    const P: u8 = 0xBC;
    type SV = SmallVec<u64, 4, U8, Pattern<P>>;
    let mut v = SV::from_iter(0..4);
    assert!(v.is_local());
    assert_eq!(v, [0, 1, 2, 3]);
    check_spare_memory(&v, P);
    let shadow = Vec::from_iter(v.drain(..));
    assert!(v.is_empty());
    check_spare_memory(&v, P);
    assert_eq!(shadow, [0, 1, 2, 3]);
}

#[test]
fn test_drain_all_heap() {
    const P: u8 = 0xBC;
    type SV = SmallVec<u64, 2, U8, Pattern<P>>;
    let mut v = SV::from_iter(0..4);
    assert!(v.is_heap());
    assert_eq!(v, [0, 1, 2, 3]);
    check_spare_memory(&v, P);
    let shadow = Vec::from_iter(v.drain(..));
    assert!(v.is_empty());
    check_spare_memory(&v, P);
    assert_eq!(shadow, [0, 1, 2, 3]);
}

#[test]
fn test_drain_prefix_local() {
    const P: u8 = 0xAB;
    type SV = SmallVec<u64, 4, U8, Pattern<P>>;
    let mut v = SV::new();
    v.extend(1..=4);
    assert!(v.is_local());
    assert_eq!(v, [1, 2, 3, 4]);
    check_spare_memory(&v, P);
    v.drain(0..2);
    assert_eq!(v, [3, 4]);
    check_spare_memory(&v, P);
}

#[test]
fn test_drain_middle_local() {
    const P: u8 = 0xBC;
    type SV = SmallVec<u64, 4, U8, Pattern<P>>;
    let mut v = SV::new();
    v.extend(1..=4);
    assert!(v.is_local());
    assert_eq!(v, [1, 2, 3, 4]);
    check_spare_memory(&v, P);
    v.drain(1..3);
    assert_eq!(v, [1, 4]);
    check_spare_memory(&v, P);
}

#[test]
fn test_drain_suffix_local() {
    const P: u8 = 0xCD;
    type SV = SmallVec<u64, 4, U8, Pattern<P>>;
    let mut v = SV::new();
    v.extend(1..=4);
    assert!(v.is_local());
    assert_eq!(v, [1, 2, 3, 4]);
    check_spare_memory(&v, P);
    v.drain(2..4);
    assert_eq!(v, [1, 2]);
    check_spare_memory(&v, P);
}

#[test]
fn test_drain_prefix_heap() {
    const P: u8 = 0xAB;
    type SV = SmallVec<u64, 1, U8, Pattern<P>>;
    let mut v = SV::new();
    v.extend(1..=4);
    assert!(v.is_heap());
    assert_eq!(v, [1, 2, 3, 4]);
    check_spare_memory(&v, P);
    v.drain(0..2);
    assert_eq!(v, [3, 4]);
    check_spare_memory(&v, P);
}

#[test]
fn test_drain_middle_heap() {
    const P: u8 = 0xBC;
    type SV = SmallVec<u64, 1, U8, Pattern<P>>;
    let mut v = SV::new();
    v.extend(1..=4);
    assert!(v.is_heap());
    assert_eq!(v, [1, 2, 3, 4]);
    check_spare_memory(&v, P);
    v.drain(1..3);
    assert_eq!(v, [1, 4]);
    check_spare_memory(&v, P);
}

#[test]
fn test_drain_suffix_heap() {
    const P: u8 = 0xCD;
    type SV = SmallVec<u64, 1, U8, Pattern<P>>;
    let mut v = SV::new();
    v.extend(1..=4);
    assert!(v.is_heap());
    assert_eq!(v, [1, 2, 3, 4]);
    check_spare_memory(&v, P);
    v.drain(2..4);
    assert_eq!(v, [1, 2]);
    check_spare_memory(&v, P);
}

#[test]
fn test_clone_local() {
    const P: u8 = 0xFA;
    type SV = SmallVec<u64, 8, U8, Pattern<P>>;
    let s = SV::from_iter(0..8);
    assert!(s.is_local());
    assert_eq!(s, [0, 1, 2, 3, 4, 5, 6, 7]);
    check_spare_memory(&s, P);
    let d = s.clone();
    assert_eq!(d, [0, 1, 2, 3, 4, 5, 6, 7]);
    check_spare_memory(&d, P);
}

#[test]
fn test_clone_heap() {
    const P: u8 = 0xFA;
    type SV = SmallVec<u64, 2, U8, Pattern<P>>;
    let s = SV::from_iter(0..8);
    assert!(s.is_heap());
    assert_eq!(s, [0, 1, 2, 3, 4, 5, 6, 7]);
    check_spare_memory(&s, P);
    let d = s.clone();
    assert_eq!(d, [0, 1, 2, 3, 4, 5, 6, 7]);
    check_spare_memory(&d, P);
}

#[test]
fn test_clone_zst() {
    gen_dropped_zst!(T);
    type SV = SmallVec<T, 8, U8>;
    let mut s = SV::new();
    s.resize_with(16, T::new);
    assert_eq!(s.len(), 16);
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 16,
            clone: 0,
            drop: 0
        }
    );
    let d = s.clone();
    assert_eq!(d.len(), 16);
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 16,
            clone: 16,
            drop: 0
        }
    );
    drop(s);
    drop(d);
    assert_eq!(
        counters::<T>(),
        Counters {
            new: 16,
            clone: 16,
            drop: 32
        }
    );
}

#[test]
fn test_clone_dropped() {
    const P: u8 = 0xFF;
    type Item<'a> = Dropped<'a, 512>;
    type SV<'a> = SmallVec<Item<'a>, 16, U8, Pattern<P>>;
    let t = Track::new();

    let v = SV::from_iter(t.take(10));
    assert_eq!(t.n_allocated(), 10);
    assert!(t.dropped_range(0..0));
    check_spare_memory(&v, P);

    let d = v.clone();
    assert_eq!(t.n_allocated(), 20);
    assert!(t.dropped_range(0..0));
    check_spare_memory(&d, P);

    drop(d);
    assert!(t.dropped_range(10..20));
}

#[test]
fn test_clone_from_local_local() {
    const P: u8 = 0xFA;
    type SV = SmallVec<u64, 8, U8, Pattern<P>>;

    let s = SV::from_iter(6..9);
    assert!(s.is_local());
    assert_eq!(s, [6, 7, 8]);

    let mut d = SV::from_iter(0..6);
    assert!(d.is_local());
    assert_eq!(d, [0, 1, 2, 3, 4, 5]);

    d.clone_from(&s);
    assert!(d.is_local());
    assert_eq!(d, [6, 7, 8]);
    check_spare_memory(&d, P);
}

#[test]
fn test_clone_from_heap_local() {
    const P: u8 = 0xFA;
    type SV = SmallVec<u64, 8, U8, Pattern<P>>;

    let s = SV::from_iter(6..15);
    assert!(s.is_heap());
    assert_eq!(s, [6, 7, 8, 9, 10, 11, 12, 13, 14]);

    let mut d = SV::from_iter(0..6);
    assert!(d.is_local());
    assert_eq!(d, [0, 1, 2, 3, 4, 5]);

    d.clone_from(&s);
    assert!(d.is_heap());
    assert_eq!(d, [6, 7, 8, 9, 10, 11, 12, 13, 14]);
    check_spare_memory(&d, P);
}

#[test]
fn test_clone_from_local_heap() {
    const P: u8 = 0xFA;
    type SV = SmallVec<u64, 8, U8, Pattern<P>>;

    let s = SV::from_iter(6..9);
    assert!(s.is_local());
    assert_eq!(s, [6, 7, 8]);

    let mut d = SV::from_iter(0..10);
    assert!(d.is_heap());
    assert_eq!(d, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    d.clone_from(&s);
    assert!(d.is_heap());
    assert_eq!(d, [6, 7, 8]);
    check_spare_memory(&d, P);
}

#[test]
fn test_clone_from_heap_heap() {
    const P: u8 = 0xFA;
    type SV = SmallVec<u64, 8, U8, Pattern<P>>;

    let s = SV::from_iter(10..40);
    assert!(s.is_heap());
    assert_eq!(s.len(), 30);

    let mut d = SV::from_iter(0..10);
    assert!(d.is_heap());
    assert_eq!(d, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    assert!(s.len() > d.capacity());

    d.clone_from(&s);
    assert!(d.is_heap());
    assert_eq!(d.as_slice(), s.as_slice());
    check_spare_memory(&d, P);
}

#[test]
fn test_spare_capacity_zst() {
    type SV = SmallVec<(), 10, U8>;
    let mut v = SV::new();
    assert_eq!(v.len(), 0);
    assert_eq!(v.spare_capacity(), v.capacity());

    for _ in 0..u8::MAX {
        v.push(());
        assert_eq!(v.spare_capacity(), v.capacity() - v.len());
    }

    assert_eq!(v.spare_capacity(), 0);
}

#[test]
fn test_spare_capacity() {
    type SV = SmallVec<u64, 10, U8>;
    let mut v = SV::new();
    assert_eq!(v.len(), 0);
    assert_eq!(v.spare_capacity(), v.capacity());

    for i in 0..u8::MAX {
        v.push(i as u64);
        assert_eq!(v.spare_capacity(), v.capacity() - v.len());
    }

    assert_eq!(v.spare_capacity(), 0);
}

#[test]
fn test_copy_from_slice_zst() {
    type SV = SmallVec<(), 8, U8>;
    let mut v = SV::new();
    assert_eq!(v.len(), 0);
    let mut src = Vec::new();
    src.resize_with(10, || ());
    assert_eq!(src.len(), 10);
    v.copy_from_slice(src.as_slice());
    assert_eq!(v.len(), 10);
}

#[test]
#[should_panic]
fn test_copy_from_slice_zst_panics() {
    type SV = SmallVec<(), 8, U8>;
    let mut v = SV::new();
    assert_eq!(v.len(), 0);
    let mut src = Vec::new();
    src.resize_with(256, || ());
    assert_eq!(src.len(), 256);
    v.copy_from_slice(src.as_slice());
}

#[test]
fn test_copy_from_slice() {
    type SV = SmallVec<u64, 10, U8>;
    let mut v = SV::new();
    v.copy_from_slice(&[1, 2]);
    assert_eq!(v, [1, 2]);

    let mut v = SV::from_iter(0..3);
    v.copy_from_slice(&[3, 4, 5]);
    assert_eq!(v, [0, 1, 2, 3, 4, 5]);

    let mut v = SV::from_iter(0..9);
    v.copy_from_slice(&[9, 10, 11]);
    assert_eq!(v, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
}

#[test]
#[should_panic]
fn test_copy_from_slice_panics() {
    type SV = SmallVec<u64, 10, U8>;
    let mut v = SV::from_iter(0..10);
    assert_eq!(v.len(), 10);
    let src = Vec::from_iter(0..255);
    v.copy_from_slice(src.as_slice());
}

#[test]
fn test_try_copy_from_slice() {
    type SV = SmallVec<u64, 8, U8>;
    let mut v = SV::new();
    v.try_copy_from_slice(&[3, 4]).unwrap();
    assert_eq!(v, [3, 4]);

    let mut v = SV::new();
    v.try_copy_from_slice(Vec::from_iter(0..8).as_slice())
        .unwrap();
    assert_eq!(v, [0, 1, 2, 3, 4, 5, 6, 7]);

    let mut v = SV::from_iter(5..8);
    v.try_copy_from_slice(Vec::from_iter(8..18).as_slice())
        .unwrap();
    assert_eq!(v, [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17]);

    let mut v = SV::from_iter(0..3);
    assert_eq!(v, [0, 1, 2]);
    assert!(matches!(
        v.try_copy_from_slice(Vec::from_iter(0..u8::MAX as u64).as_slice()),
        Err(ReservationError::CapacityOverflow)
    ));
}

#[test]
fn test_try_copy_from_slice_zst() {
    type SV = SmallVec<(), 8, U8>;
    let mut v = SV::new();

    let mut src = Vec::new();
    src.resize_with(10, || ());

    v.try_copy_from_slice(src.as_slice()).unwrap();
    assert_eq!(v.len(), src.len());

    src.resize_with(246, || ());
    assert!(matches!(
        v.try_copy_from_slice(src.as_slice()),
        Err(ReservationError::CapacityOverflow)
    ));
}

#[test]
fn test_is_full_zst() {
    type SV = SmallVec<(), 8, U8>;
    let mut v = SV::new();
    assert!(!v.is_full());

    for i in 0..u8::MAX {
        v.push(());
        assert_eq!(v.len(), (i + 1) as usize);
        assert_eq!(v.is_full(), (i + 1) == u8::MAX);
    }
}

#[test]
fn test_is_full() {
    type SV = SmallVec<usize, 8, U8>;
    let mut v = SV::new();
    assert!(!v.is_full());

    for i in 0..u8::MAX {
        v.push(i as usize);
        assert_eq!(v.len(), (i + 1) as usize);
        assert_eq!(
            v.is_full(),
            (i > 3) && ((i == 7) || (i == 254) || (i + 1).is_power_of_two())
        );
    }
}

#[test]
fn test_has_spare_capacity_zst() {
    type SV = SmallVec<(), 8, U8>;
    let mut v = SV::new();
    assert!(v.has_spare_capacity());

    for i in 0..255 {
        v.push(());
        assert_eq!(v.has_spare_capacity(), i < 254);
    }
}

#[test]
fn test_has_spare_capacity() {
    type SV = SmallVec<usize, 8, U8>;
    let mut v = SV::new();
    assert!(v.has_spare_capacity());

    for i in 0..255 {
        v.push(i);
        assert_eq!(
            v.has_spare_capacity(),
            (i != 7) && (i != 254) && (i <= 3 || !(i + 1).is_power_of_two())
        );
    }
}

#[test]
fn test_spare_capacity_mut() {
    const P: u8 = 0xBA;
    type SV = SmallVec<u64, 8, U8, Pattern<P>>;
    let mut v = SV::new();
    let sc = v.spare_capacity_mut();
    assert_eq!(sc.len(), 8);
    for e in sc {
        assert_eq!(unsafe { e.as_ptr().read() }, 0xBABABABABABABABA);
    }

    for i in 0..255usize {
        v.push(i as u64);
        let sc = v.spare_capacity_mut();
        let capacity = if i < 8 {
            8
        } else if i > 127 {
            255
        } else {
            // the smallest power of two >= self
            (i + 1).next_power_of_two()
        };
        assert_eq!(sc.len(), capacity - i - 1);
        for e in sc {
            assert_eq!(unsafe { e.as_ptr().read() }, 0xBABABABABABABABA);
        }
        assert_eq!(v, Vec::from_iter(0..(i + 1) as u64).as_slice());
    }

    assert_eq!(v.spare_capacity_mut().len(), 0);
}

#[test]
fn test_spare_capacity_mut_zst() {
    type SV = SmallVec<(), 8, U8>;
    let mut v = SV::new();
    let p = v.as_ptr();
    assert_eq!(v.spare_capacity_mut().len(), 255);
    for i in 0..255 {
        v.push(());
        assert_eq!(v.spare_capacity_mut().len(), 255 - i - 1);
        assert_eq!(v.spare_capacity_mut().as_ptr(), p.cast());
    }
    assert_eq!(v.spare_capacity_mut().len(), 0);
}

#[test]
fn test_split_at_spare_mut() {
    type SV = SmallVec<usize, 8, U8>;
    let mut v = SV::new();

    {
        let (d, s) = v.split_at_spare_mut();
        assert_eq!(d.len(), 0);
        assert_eq!(s.len(), 8);
    }

    for i in 0..255 {
        v.push(i);
        let capacity: usize = if i < 8 {
            8
        } else if i > 127 {
            255
        } else {
            // the smallest power of two >= self
            (i + 1).next_power_of_two()
        };
        let (d, s) = v.split_at_spare_mut();
        assert_eq!(d, Vec::from_iter(0..=i).as_slice());
        assert_eq!(s.len(), capacity - i - 1);
    }

    {
        let (d, s) = v.split_at_spare_mut();
        assert_eq!(d.len(), 255);
        assert_eq!(s.len(), 0);
    }
}

#[test]
fn test_split_at_spare_mut_zst() {
    type SV = SmallVec<(), 8, U8>;
    let mut v = SV::new();

    {
        let (d, s) = v.split_at_spare_mut();
        assert_eq!(d.len(), 0);
        assert_eq!(s.len(), 255);
    }

    for i in 0..255 {
        v.push(());
        let (d, s) = v.split_at_spare_mut();
        assert_eq!(d.len(), i + 1);
        assert_eq!(s.len(), 255 - i - 1);
    }

    {
        let (d, s) = v.split_at_spare_mut();
        assert_eq!(d.len(), 255);
        assert_eq!(s.len(), 0);
    }
}

#[test]
#[allow(dead_code)]
fn test_small_vec_covariance() {
    fn foo<'a>(v: SmallVec<&'static str, 8>) -> SmallVec<&'a str, 8> {
        v
    }
}

#[test]
#[allow(dead_code)]
fn test_small_vec_drain_covariance() {
    fn foo<'a>(
        d: Drain<'a, &'static str, Usize, Uninitialized, 8>,
    ) -> Drain<'a, &'a str, Usize, Uninitialized, 8> {
        d
    }
}
