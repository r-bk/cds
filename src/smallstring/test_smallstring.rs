use crate as cds;
use crate::{
    len::{LengthType, U8},
    mem::{Pattern, SpareMemoryPolicy},
    small_str,
    smallstring::SmallString,
};

pub(crate) fn check_spare_memory<L, SM, const C: usize>(s: &SmallString<C, L, SM>, pattern: u8)
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    let sc = s.spare_capacity();

    assert_eq!(sc.len(), s.capacity() - s.len());

    for b in sc {
        assert_eq!(unsafe { b.assume_init() }, pattern);
    }
}

#[test]
fn test_new() {
    type S = SmallString<16, U8, Pattern<0xBA>>;
    let s = S::new();
    assert_eq!(s, "");
    assert_eq!(s.len(), 0);
    assert!(s.is_empty());
    check_spare_memory(&s, 0xBA);
}

#[test]
fn test_with_capacity() {
    type S = SmallString<16, U8, Pattern<0xDC>>;

    let s = S::with_capacity(8);
    assert_eq!(s.capacity(), 16);
    check_spare_memory(&s, 0xDC);

    let s = S::with_capacity(16);
    assert_eq!(s.capacity(), 16);
    check_spare_memory(&s, 0xDC);

    let s = S::with_capacity(17);
    assert_eq!(s.capacity(), 17);
    check_spare_memory(&s, 0xDC);
}

#[test]
fn test_reserve() {
    type S = SmallString<4, U8, Pattern<0xAA>>;
    let mut s = S::new();
    assert_eq!(s.capacity(), 4);
    check_spare_memory(&s, 0xAA);
    s.reserve(4);
    assert_eq!(s.capacity(), 4);
    s.reserve(8);
    assert_eq!(s.capacity(), 8);
    check_spare_memory(&s, 0xAA);
    s.reserve(9);
    assert_eq!(s.capacity(), 16);
    check_spare_memory(&s, 0xAA);
    s.reserve(16);
    assert_eq!(s.capacity(), 16);
    s.reserve(17);
    assert_eq!(s.capacity(), 32);
    check_spare_memory(&s, 0xAA);

    s.push_str("0123456789");
    s.push_str("0123456789");
    assert_eq!(s.capacity(), 32);
    s.reserve(17);
    assert_eq!(s.capacity(), 64);
    check_spare_memory(&s, 0xAA);
}

#[test]
fn test_reserve_exact() {
    type S = SmallString<4, U8, Pattern<0xAA>>;

    let mut s = S::new();
    assert_eq!(s.capacity(), 4);
    check_spare_memory(&s, 0xAA);

    s.reserve_exact(4);
    assert_eq!(s.capacity(), 4);
    check_spare_memory(&s, 0xAA);

    s.reserve_exact(5);
    assert_eq!(s.capacity(), 5);
    check_spare_memory(&s, 0xAA);

    s.reserve_exact(20);
    assert_eq!(s.capacity(), 20);
    check_spare_memory(&s, 0xAA);

    s.push_str("0123456789");
    s.push_str("0123456789");
    assert_eq!(s, "01234567890123456789");
    assert_eq!(s.capacity(), 20);
    check_spare_memory(&s, 0xAA);

    s.reserve_exact(1);
    assert_eq!(s, "01234567890123456789");
    assert_eq!(s.capacity(), 21);
    check_spare_memory(&s, 0xAA);
}

#[test]
fn test_push_str() {
    type S = SmallString<4, U8, Pattern<0xAA>>;

    let mut s = S::new();
    assert_eq!(s.capacity(), 4);
    assert_eq!(s, "");
    check_spare_memory(&s, 0xAA);

    s.push_str("");
    assert_eq!(s.capacity(), 4);
    assert_eq!(s, "");
    check_spare_memory(&s, 0xAA);

    s.push_str("ab");
    assert_eq!(s.capacity(), 4);
    assert_eq!(s, "ab");
    check_spare_memory(&s, 0xAA);

    s.push_str("cd");
    assert_eq!(s.capacity(), 4);
    assert_eq!(s, "abcd");
    check_spare_memory(&s, 0xAA);

    s.push_str("a");
    assert_eq!(s.capacity(), 8);
    assert_eq!(s, "abcda");
    check_spare_memory(&s, 0xAA);
}

#[test]
fn test_push() {
    type S = SmallString<4, U8, Pattern<0xAA>>;

    let mut s = S::new();
    assert_eq!(s, "");
    check_spare_memory(&s, 0xAA);

    s.push('a');
    assert_eq!(s, "a");
    assert_eq!(s.capacity(), 4);
    check_spare_memory(&s, 0xAA);

    s.push('b');
    assert_eq!(s, "ab");
    assert_eq!(s.capacity(), 4);
    check_spare_memory(&s, 0xAA);

    s.push('€');
    assert_eq!(s, "ab€");
    assert_eq!(s.len(), 5);
    assert_eq!(s.capacity(), 8);
    check_spare_memory(&s, 0xAA);

    s.push('€');
    s.push('€');
    assert_eq!(s, "ab€€€");
    assert_eq!(s.len(), 11);
    assert_eq!(s.capacity(), 16);
    check_spare_memory(&s, 0xAA);

    s.clear();
    check_spare_memory(&s, 0xAA);

    s.push('€');
    assert_eq!(s, "€");
    check_spare_memory(&s, 0xAA);
}

#[test]
fn test_pop() {
    type S = SmallString<4, U8, Pattern<0xCD>>;

    let mut s = S::new();
    s.push_str("€ab€");
    check_spare_memory(&s, 0xCD);

    assert_eq!(s.pop(), Some('€'));
    assert_eq!(s, "€ab");
    check_spare_memory(&s, 0xCD);

    assert_eq!(s.pop(), Some('b'));
    assert_eq!(s, "€a");
    check_spare_memory(&s, 0xCD);

    assert_eq!(s.pop(), Some('a'));
    assert_eq!(s, "€");
    check_spare_memory(&s, 0xCD);

    assert_eq!(s.pop(), Some('€'));
    assert_eq!(s, "");
    check_spare_memory(&s, 0xCD);

    assert_eq!(s.pop(), None);

    let mut s = S::new();
    s.push_str("ab");
    check_spare_memory(&s, 0xCD);

    assert_eq!(s.pop(), Some('b'));
    assert_eq!(s, "a");
    check_spare_memory(&s, 0xCD);

    assert_eq!(s.pop(), Some('a'));
    assert_eq!(s, "");
    check_spare_memory(&s, 0xCD);

    assert_eq!(s.pop(), None);
}

#[test]
#[should_panic]
fn test_insert_panics_on_out_of_bounds() {
    let mut s = small_str![4; "abc"];
    s.insert(4, 'd');
}

#[test]
#[should_panic]
fn test_insert_panics_on_no_character_boundary() {
    let mut s = small_str![4; "2€"];
    s.insert(2, '+');
}

#[test]
fn test_insert() {
    type S = SmallString<4, U8, Pattern<0xCD>>;
    let expected = "€0123456789";

    let mut s = S::new();
    let mut compared = 0;
    for c in expected.chars().rev() {
        s.insert(0, c);
        compared += c.len_utf8();
        assert_eq!(s, expected[(expected.len() - compared)..]);
        check_spare_memory(&s, 0xCD);
    }
}

#[test]
#[should_panic]
fn test_insert_str_panics_on_out_of_bounds() {
    let mut s = small_str![8; "abc"];
    s.insert_str(4, "def");
}

#[test]
#[should_panic]
fn test_insert_str_panics_on_no_character_boundary() {
    let mut s = small_str![4; "2€"];
    s.insert_str(2, "+");
}

#[test]
fn test_insert_str() {
    type S = SmallString<4, U8, Pattern<0xCD>>;
    let expected = "€0123456789";

    let mut s = S::new();
    let mut compared = 0;
    for c in expected.chars().rev() {
        use alloc::string::ToString;
        s.insert_str(0, c.to_string().as_str());
        compared += c.len_utf8();
        assert_eq!(s, expected[(expected.len() - compared)..]);
        check_spare_memory(&s, 0xCD);
    }
}

#[test]
#[should_panic]
fn test_truncate_panics_local() {
    let mut s = small_str![8; "2€"];
    s.truncate(2);
}

#[test]
#[should_panic]
fn test_truncate_panics_local_full() {
    let mut s = small_str![4; "2€"];
    s.truncate(2);
}

#[test]
#[should_panic]
fn test_truncate_panics_heap() {
    let mut s = small_str![2; "2€"];
    s.truncate(2);
}

#[test]
fn test_truncate_larger() {
    let mut s = small_str![8; "cds"];
    assert_eq!(s, "cds");
    s.truncate(4);
    assert_eq!(s, "cds");

    let mut s = small_str![3; "cds"];
    assert_eq!(s, "cds");
    s.truncate(4);
    assert_eq!(s, "cds");

    let mut s = small_str![2; "cds"];
    assert_eq!(s, "cds");
    s.truncate(4);
    assert_eq!(s, "cds");
}

#[test]
fn test_truncate_equal() {
    let mut s = small_str![4; "cds"];
    assert_eq!(s, "cds");
    s.truncate(s.len());
    assert_eq!(s, "cds");

    let mut s = small_str![3; "cds"];
    assert_eq!(s, "cds");
    s.truncate(s.len());
    assert_eq!(s, "cds");

    let mut s = small_str![2; "cds"];
    assert_eq!(s, "cds");
    s.truncate(s.len());
    assert_eq!(s, "cds");
}

#[test]
fn test_truncate() {
    type S = SmallString<4, U8, Pattern<0xCD>>;
    for slc in ["abcde", "abcd", "abc"] {
        let mut s = S::new();
        check_spare_memory(&s, 0xCD);
        s.push_str(slc);
        check_spare_memory(&s, 0xCD);
        s.truncate(2);
        assert_eq!(s, "ab");
        check_spare_memory(&s, 0xCD);
    }
}

#[test]
#[should_panic]
fn test_remove_panics_beyond() {
    let mut s = small_str![8; "cds"];
    s.remove(4);
}

#[test]
#[should_panic]
fn test_remove_panics_at() {
    let mut s = small_str![8; "cds"];
    s.remove(3);
}

#[test]
#[should_panic]
fn test_remove_panics_on_no_character_boundary() {
    let mut s = small_str![8; "100€"];
    s.remove(4);
}

#[test]
fn test_remove_last() {
    {
        type S = SmallString<4, U8, Pattern<0xCD>>;
        let mut s = S::new();
        s.push_str("0€");
        check_spare_memory(&s, 0xCD);
        assert_eq!(s.remove(1), '€');
        assert_eq!(s, "0");
        check_spare_memory(&s, 0xCD);
    }
    {
        type S = SmallString<2, U8, Pattern<0xCD>>;
        let mut s = S::new();
        s.push_str("0€");
        check_spare_memory(&s, 0xCD);
        assert_eq!(s.remove(1), '€');
        assert_eq!(s, "0");
        check_spare_memory(&s, 0xCD);
    }
}

#[test]
fn test_remove_middle() {
    {
        type S = SmallString<8, U8, Pattern<0xCD>>;
        let mut s = S::new();
        s.push_str("0€0");
        check_spare_memory(&s, 0xCD);
        assert_eq!(s.remove(1), '€');
        assert_eq!(s, "00");
        check_spare_memory(&s, 0xCD);
    }
    {
        type S = SmallString<2, U8, Pattern<0xCD>>;
        let mut s = S::new();
        s.push_str("0€0");
        check_spare_memory(&s, 0xCD);
        assert_eq!(s.remove(1), '€');
        assert_eq!(s, "00");
        check_spare_memory(&s, 0xCD);
    }
}

#[test]
fn test_remove_first() {
    {
        type S = SmallString<8, U8, Pattern<0xCD>>;
        let mut s = S::new();
        s.push_str("€00");
        check_spare_memory(&s, 0xCD);
        assert_eq!(s.remove(0), '€');
        assert_eq!(s, "00");
        check_spare_memory(&s, 0xCD);
    }
    {
        type S = SmallString<2, U8, Pattern<0xCD>>;
        let mut s = S::new();
        s.push_str("€00");
        check_spare_memory(&s, 0xCD);
        assert_eq!(s.remove(0), '€');
        assert_eq!(s, "00");
        check_spare_memory(&s, 0xCD);
    }
}
