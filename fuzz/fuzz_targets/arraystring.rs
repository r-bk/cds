#![no_main]
use cds::{len::U8, mem::Pattern};
use libfuzzer_sys::{arbitrary, fuzz_target};

const CAP: usize = 255;
const PATTERN: u8 = 0xBE;

type ArrayString = cds::arraystring::ArrayString<U8, Pattern<PATTERN>, CAP>;

#[derive(arbitrary::Arbitrary, Debug)]
enum Op {
    TryPush(char),
    TryPushStr(String),
    Pop,
    Insert(u8, char),
    InsertStr(u8, String),
    Remove(u8),
    Truncate(u8),
    Clear,
    CompareShadow,
    CheckSpareMemory,
}

fuzz_target!(|ops: Vec<Op>| {
    let mut s = ArrayString::new();
    let mut shadow = String::with_capacity(CAP);

    for op in ops {
        match op {
            Op::TryPush(ch) => {
                s.try_push(ch).ok();
                if ch.len_utf8() <= (CAP - shadow.len()) {
                    shadow.push(ch);
                }
            }
            Op::TryPushStr(sl) => {
                s.try_push_str(&sl).ok();
                if sl.len() <= (CAP - shadow.len()) {
                    shadow.push_str(&sl);
                }
            }
            Op::Pop => {
                assert_eq!(s.pop(), shadow.pop());
            }
            Op::Insert(idx, ch) => {
                let idx = (idx as usize) % (s.len() + 1);
                if ch.len_utf8() <= s.spare_capacity() && s.is_char_boundary(idx) {
                    s.insert(idx, ch);
                    shadow.insert(idx, ch);
                }
            }
            Op::InsertStr(idx, sl) => {
                let idx = (idx as usize) % (s.len() + 1);
                if sl.len() <= s.spare_capacity() && s.is_char_boundary(idx) {
                    s.insert_str(idx, &sl);
                    shadow.insert_str(idx, &sl);
                }
            }
            Op::Remove(idx) => {
                let idx = (idx as usize) % (s.len() + 1);
                if s.is_char_boundary(idx) && idx < s.len() {
                    assert_eq!(s.remove(idx), shadow.remove(idx));
                }
            }
            Op::Truncate(idx) => {
                let idx = (idx as usize) % (s.len() + 1);
                if idx >= s.len() || s.is_char_boundary(idx) {
                    s.truncate(idx);
                    shadow.truncate(idx);
                }
            }
            Op::Clear => {
                s.clear();
                shadow.clear();
            }
            Op::CompareShadow => {
                assert_eq!(*s, *shadow);
            }
            Op::CheckSpareMemory => unsafe {
                let mut p = s.as_ptr().add(s.len());
                let end = s.as_ptr().add(CAP);
                while p < end {
                    assert_eq!(p.read(), PATTERN);
                    p = p.add(1);
                }
            },
        }
    }
});
