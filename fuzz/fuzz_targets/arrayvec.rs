#![no_main]
use cds::defs::{Pattern, U8};
use libfuzzer_sys::{arbitrary, fuzz_target};

const TRACK_SIZE: usize = 128;
const AV_SIZE: usize = 64;
const PATTERN: u8 = 0xBE;

type Track = cds_fuzz::Track<TRACK_SIZE>;
type Element<'a> = cds_fuzz::Element<'a, TRACK_SIZE>;
type ArrayVec<'a> = cds::arrayvec::ArrayVec<Element<'a>, U8, Pattern<PATTERN>, AV_SIZE>;

#[derive(arbitrary::Arbitrary, Debug)]
enum Op {
    Push,
    TryPush,
    TryPushVal,
    Pop,
    Insert(u8),
    TryInsert(u8),
    TryInsertVal(u8),
    Remove(u8),
    TryRemove(u8),
    SwapRemove(u8),
    TrySwapRemove(u8),
    Truncate(u8),
    Clear,
    Drain(u8, u8, bool),
    CompareShadow,
    CheckSpareMemory,
    CheckDropped,
}

fuzz_target!(|ops: Vec<Op>| {
    let t = Track::new();
    let mut av = ArrayVec::new();
    let mut shadow = Vec::with_capacity(AV_SIZE);

    for op in ops {
        match op {
            Op::Push => {
                if av.has_spare_capacity() {
                    let e = t.alloc();
                    shadow.push(e.idx());
                    av.push(e);
                }
            }
            Op::TryPush => {
                let e = t.alloc();
                let idx = e.idx();
                if av.try_push(e).is_ok() {
                    shadow.push(idx);
                }
            }
            Op::TryPushVal => {
                let e = t.alloc();
                let idx = e.idx();
                if av.try_push_val(e).is_ok() {
                    shadow.push(idx);
                }
            }
            Op::Pop => {
                let se = shadow.pop();
                let e = av.pop();
                if let Some(v) = e {
                    assert_eq!(v.idx(), se.unwrap());
                } else {
                    assert!(se.is_none());
                }
            }
            Op::Insert(i) => {
                if av.has_spare_capacity() {
                    let index = (i as usize) % (av.len() + 1);
                    let e = t.alloc();
                    shadow.insert(index, e.idx());
                    av.insert(index, e);
                }
            }
            Op::TryInsert(i) => {
                let e = t.alloc();
                let idx = e.idx();
                let rc = av.try_insert(i as usize, e);
                if rc.is_ok() {
                    shadow.insert(i as usize, idx);
                }
            }
            Op::TryInsertVal(i) => {
                let e = t.alloc();
                let idx = e.idx();
                let rc = av.try_insert_val(i as usize, e);
                if rc.is_ok() {
                    shadow.insert(i as usize, idx);
                }
            }
            Op::Remove(i) => {
                if !av.is_empty() {
                    let index = (i as usize) % av.len();
                    assert_eq!(av.remove(index).idx(), shadow.remove(index));
                }
            }
            Op::TryRemove(i) => {
                if let Some(v) = av.try_remove(i as usize) {
                    assert_eq!(v.idx(), shadow.remove(i as usize));
                }
            }
            Op::SwapRemove(i) => {
                if !av.is_empty() {
                    let index = (i as usize) % av.len();
                    assert_eq!(av.swap_remove(index).idx(), shadow.swap_remove(index));
                }
            }
            Op::TrySwapRemove(i) => {
                if let Some(v) = av.try_swap_remove(i as usize) {
                    assert_eq!(v.idx(), shadow.swap_remove(i as usize));
                }
            }
            Op::Truncate(l) => {
                av.truncate(l as usize);
                shadow.truncate(l as usize);
            }
            Op::Clear => {
                av.clear();
                shadow.clear();
            }
            Op::Drain(i, l, consume) => {
                if !av.is_empty() {
                    let start = (i as usize) % av.len();
                    let end = (start + l as usize).min(av.len());
                    let mut dav = av.drain(start..end);
                    let mut ds = shadow.drain(start..end);

                    if consume {
                        loop {
                            if let Some(v) = dav.next() {
                                assert_eq!(v.idx(), ds.next().unwrap());
                            } else {
                                assert!(ds.next().is_none());
                                break;
                            }
                        }
                    }
                }
            }
            Op::CompareShadow => {
                assert_eq!(av.len(), shadow.len());
                for i in 0..av.len() {
                    assert_eq!(av[i].idx(), shadow[i]);
                }
            }
            Op::CheckSpareMemory => unsafe {
                let mut p = av.as_ptr().add(av.len()) as *const u8;
                let end = av.as_ptr().add(av.capacity()) as *const u8;
                while p < end {
                    assert_eq!(p.read(), PATTERN);
                    p = p.add(1);
                }
            },
            Op::CheckDropped => {
                assert!(t.is_allocated_exact(&shadow));
            }
        }
    }
});
