#![no_main]
use cds::{len::Usize, mem::Pattern};
use libfuzzer_sys::{arbitrary, fuzz_target};

const TRACK_SIZE: usize = 2048;
const SV_SIZE: usize = 8;
const SV_MAX_SIZE: usize = 1024;
const PATTERN: u8 = 0xBE;

type Track = cds_fuzz::Track<TRACK_SIZE>;
type Element<'a> = cds_fuzz::Element<'a, TRACK_SIZE>;
type SmallVec<'a> = cds::smallvec::SmallVec<Element<'a>, SV_SIZE, Usize, Pattern<PATTERN>>;

#[derive(arbitrary::Arbitrary, Debug)]
enum Op {
    Push,
    TryPush,
    Pop,
    Insert(u8),
    TryInsert(u8),
    Remove(u8),
    TryRemove(u8),
    SwapRemove(u8),
    TrySwapRemove(u8),
    Truncate(u8),
    Clear,
    Drain(u8, u8, bool),
    IndexMut(u8),
    Extend(u8),
    Retain(u8),
    ResizeWith(u8),
    CompareShadow,
    CheckSpareMemory,
    CheckDropped,
}

fuzz_target!(|ops: Vec<Op>| {
    let t = Track::new();
    let mut sv = SmallVec::new();
    let mut shadow = Vec::with_capacity(SV_MAX_SIZE);

    for op in ops {
        match op {
            Op::Push => {
                let e = t.alloc();
                shadow.push(e.idx());
                sv.push(e);
            }
            Op::TryPush => {
                let e = t.alloc();
                shadow.push(e.idx());
                sv.try_push(e).expect("try_push failed");
            }
            Op::Pop => {
                let se = shadow.pop();
                let e = sv.pop();
                if let Some(v) = e {
                    assert_eq!(v.idx(), se.unwrap());
                } else {
                    assert!(se.is_none());
                }
            }
            Op::Insert(i) => {
                let index = (i as usize) % (sv.len() + 1);
                let e = t.alloc();
                shadow.insert(index, e.idx());
                sv.insert(index, e);
            }
            Op::TryInsert(i) => {
                let e = t.alloc();
                let idx = e.idx();
                let rc = sv.try_insert(i as usize, e);
                if rc.is_ok() {
                    shadow.insert(i as usize, idx);
                }
            }
            Op::Remove(i) => {
                if !sv.is_empty() {
                    let index = (i as usize) % sv.len();
                    assert_eq!(sv.remove(index).idx(), shadow.remove(index));
                }
            }
            Op::TryRemove(i) => {
                if let Some(v) = sv.try_remove(i as usize) {
                    assert_eq!(v.idx(), shadow.remove(i as usize));
                }
            }
            Op::SwapRemove(i) => {
                if !sv.is_empty() {
                    let index = (i as usize) % sv.len();
                    assert_eq!(sv.swap_remove(index).idx(), shadow.swap_remove(index));
                }
            }
            Op::TrySwapRemove(i) => {
                if let Some(v) = sv.try_swap_remove(i as usize) {
                    assert_eq!(v.idx(), shadow.swap_remove(i as usize));
                }
            }
            Op::Truncate(l) => {
                sv.truncate(l as usize);
                shadow.truncate(l as usize);
            }
            Op::Clear => {
                sv.clear();
                shadow.clear();
            }
            Op::Drain(i, l, consume) => {
                if !sv.is_empty() {
                    let start = (i as usize) % sv.len();
                    let end = (start + l as usize).min(sv.len());
                    let mut dav = sv.drain(start..end);
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
            Op::IndexMut(i) => {
                if !sv.is_empty() {
                    let index = (i as usize) % sv.len();
                    let e = t.alloc();
                    shadow[index] = e.idx();
                    sv[index] = e;
                }
            }
            Op::Extend(n) => {
                let len = sv.len();
                if len < SV_MAX_SIZE {
                    let num = (SV_MAX_SIZE - len).min(n as usize);
                    sv.extend(t.take(num));
                    for i in len..sv.len() {
                        shadow.push(sv[i].idx());
                    }
                }
            }
            Op::Retain(n) => {
                let idx = (n as usize) % 5 + 1;
                shadow.retain(|e| e % idx != 0);
                sv.retain(|e| e.idx() % idx != 0);
            }
            Op::ResizeWith(n) => {
                let mut len = sv.len();
                sv.resize_with(n as usize, || t.alloc());
                shadow.resize_with(n as usize, || {
                    let idx = sv[len].idx();
                    len += 1;
                    idx
                })
            }
            Op::CompareShadow => {
                assert_eq!(sv.len(), shadow.len());
                for i in 0..sv.len() {
                    assert_eq!(sv[i].idx(), shadow[i]);
                }
            }
            Op::CheckSpareMemory => unsafe {
                let mut p = sv.as_ptr().add(sv.len()) as *const u8;
                let end = sv.as_ptr().add(sv.capacity()) as *const u8;
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
