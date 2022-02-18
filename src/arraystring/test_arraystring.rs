use crate as cds;
use crate::{
    array_str,
    arraystring::ArrayString,
    errors::{CapacityError, IndexError, InsertError},
    len::{LengthType, U8},
    mem::{Pattern, SpareMemoryPolicy},
};

pub(crate) fn check_spare_memory<L, SM, const C: usize>(s: &ArrayString<L, SM, C>, pattern: u8)
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    let mut p = unsafe { s.as_ptr().add(s.len()) };
    let end = unsafe { s.as_ptr().add(C) };

    while p < end {
        unsafe {
            assert_eq!(p.read(), pattern);
            p = p.add(1);
        }
    }
}

#[test]
fn test_new() {
    type S = ArrayString<U8, Pattern<0xBA>, 16>;
    let s = S::new();
    assert_eq!(s, "");
    assert_eq!(s.len(), 0);
    assert!(s.is_empty());
    check_spare_memory(&s, 0xBA);
}

#[test]
fn test_is_empty() {
    let mut s = array_str![16;];
    assert!(s.is_empty());
    s.push('c');
    s.push('d');
    s.push('s');
    assert!(!s.is_empty());
    s.clear();
    assert!(s.is_empty());
}

#[test]
fn test_clear() {
    type AS = ArrayString<U8, Pattern<0xBA>, 16>;
    let mut s = AS::try_from("cds").unwrap();
    assert_eq!(s, "cds");
    s.clear();
    assert_eq!(s, "");
    assert!(s.is_empty());
    check_spare_memory(&s, 0xBA);
}

#[test]
fn test_push_unchecked() {
    let mut s = array_str![16;];
    assert_eq!(s.len(), 0);
    assert_eq!(s, "");
    unsafe { s.push_unchecked('A') };
    unsafe { s.push_unchecked('B') };
    assert_eq!(s.len(), 2);
    assert_eq!(s, "AB");
}

#[test]
fn test_try_push() {
    let mut s = array_str![3;];
    s.try_push('a').unwrap();
    s.try_push('b').unwrap();
    s.try_push('c').unwrap();
    assert_eq!(s, "abc");
    assert_eq!(s.len(), 3);
    assert!(matches!(s.try_push('d'), Err(CapacityError)));
}

#[test]
fn test_push() {
    let mut s = array_str![3;];
    assert_eq!(s, "");
    s.push('a');
    s.push('b');
    s.push('c');
    assert_eq!(s, "abc");
}

#[test]
#[should_panic]
fn test_push_panics() {
    let mut s = array_str![0;];
    s.push('a');
}

#[test]
fn test_push_str_unchecked() {
    let mut s = array_str![8; "abc"];
    unsafe { s.push_str_unchecked("def") };
    assert_eq!(s, "abcdef");
}

#[test]
fn test_try_push_str() {
    let mut s = array_str![8; "abc"];
    s.try_push_str("def").unwrap();
    assert_eq!(s, "abcdef");
    assert!(matches!(s.try_push_str("ghi"), Err(CapacityError)));
}

#[test]
fn test_push_str() {
    let mut s = array_str![8; "abc"];
    s.push_str("def");
    assert_eq!(s, "abcdef");
}

#[test]
#[should_panic]
fn test_push_str_panics() {
    let mut s = array_str![3; "abc"];
    s.push_str("def");
}

#[test]
fn test_pop() {
    type S = ArrayString<U8, Pattern<0xAF>, 8>;
    let mut s = S::try_from("2€").unwrap();

    assert_eq!(Some('€'), s.pop());
    assert_eq!("2", s);
    check_spare_memory(&s, 0xAF);

    assert_eq!(Some('2'), s.pop());
    assert_eq!("", s);
    check_spare_memory(&s, 0xAF);

    assert_eq!(None, s.pop());
    check_spare_memory(&s, 0xAF);
}

#[test]
fn test_insert() {
    let mut s = array_str![8; "ab"];
    s.insert(2, 'c');
    assert_eq!(s, "abc");

    let mut s = array_str![8; "ab"];
    s.insert(0, 'c');
    assert_eq!(s, "cab");

    let mut s = array_str![8; "ab"];
    s.insert(1, 'c');
    assert_eq!(s, "acb");
}

#[test]
#[should_panic]
fn test_insert_no_char_boundary() {
    let mut s = array_str![8; "2€"];
    assert_eq!(s.len(), 4);
    assert!(!s.is_char_boundary(2));
    s.insert(2, '2');
}

#[test]
#[should_panic]
fn test_insert_index_out_of_bounds() {
    let mut s = array_str![8; "ab"];
    assert_eq!(s.len(), 2);
    s.insert(3, 'c');
}

#[test]
#[should_panic]
fn test_insert_no_spare_capacity() {
    let mut s = array_str![3; "2"];
    s.insert(1, '€');
}

#[test]
fn test_try_insert() {
    let mut s = array_str![6; "2"];
    s.try_insert(1, '€').unwrap();
    assert_eq!(s, "2€");

    assert!(matches!(
        s.try_insert(3, 'a'),
        Err(InsertError::InvalidIndex)
    ));

    assert!(matches!(
        s.try_insert(5, 'a'),
        Err(InsertError::InvalidIndex)
    ));

    assert!(matches!(
        s.try_insert(4, '€'),
        Err(InsertError::InsufficientCapacity)
    ));
}

#[test]
fn test_try_insert_str() {
    let mut s = array_str![4; "€"];
    assert!(s.try_insert_str(0, "2").is_ok());
    assert_eq!(s, "2€");

    assert!(matches!(
        s.try_insert_str(2, "a"),
        Err(InsertError::InvalidIndex)
    ));

    assert!(matches!(
        s.try_insert_str(5, "a"),
        Err(InsertError::InvalidIndex)
    ));

    assert!(matches!(
        s.try_insert_str(4, "a"),
        Err(InsertError::InsufficientCapacity)
    ));
}

#[test]
fn test_insert_str() {
    let mut s = array_str![3;];
    s.insert_str(0, "cds");
    assert_eq!(s, "cds");
}

#[test]
#[should_panic]
fn test_insert_str_no_char_boundary() {
    let mut s = array_str![16; "€"];
    s.insert_str(1, "a");
}

#[test]
#[should_panic]
fn test_insert_str_index_out_of_bounds() {
    let mut s = array_str![16; "cds"];
    s.insert_str(5, "a");
}

#[test]
#[should_panic]
fn test_insert_str_no_spare_capacity() {
    let mut s = array_str![5; "€"];
    s.insert_str(3, "abc");
}

#[test]
fn test_try_remove() {
    type AS = ArrayString<U8, Pattern<0xBE>, 16>;
    let mut s = AS::try_from("2€ +").unwrap();
    assert_eq!(s, "2€ +");

    assert!(matches!(s.try_remove(2), Err(IndexError))); // not char boundary
    assert!(matches!(s.try_remove(s.len()), Err(IndexError))); // index equals length
    assert!(matches!(s.try_remove(s.len() + 1), Err(IndexError))); // index exceeds length

    assert_eq!(s.try_remove(1).unwrap(), '€');
    assert_eq!(s, "2 +");

    check_spare_memory(&s, 0xBE);
}

#[test]
fn test_remove() {
    const S: &str = "cds";
    type AS = ArrayString<U8, Pattern<0xBE>, 16>;
    let mut s = AS::try_from(S).unwrap();
    for i in 0..S.len() {
        s.remove(0);
        assert_eq!(s, S[i + 1..]);
    }
    check_spare_memory(&s, 0xBE);
}

#[test]
#[should_panic]
fn test_remove_no_char_boundary() {
    let mut s = array_str![16; "2€"];
    s.remove(2);
}

#[test]
#[should_panic]
fn test_remove_index_equals_length() {
    let mut s = array_str![16; "2€"];
    s.remove(s.len());
}

#[test]
#[should_panic]
fn test_remove_index_exceeds_length() {
    let mut s = array_str![16; "2€"];
    s.remove(s.len() + 1);
}

#[test]
fn test_try_truncate() {
    type AS = ArrayString<U8, Pattern<0xEF>, 16>;
    let mut s = AS::try_from("2€").unwrap();

    assert!(matches!(s.try_truncate(2), Err(IndexError)));

    for new_len in 4..6 {
        assert!(s.try_truncate(new_len).is_ok());
        assert_eq!(s, "2€");
    }

    assert!(s.try_truncate(1).is_ok());
    assert_eq!(s, "2");
    check_spare_memory(&s, 0xEF);
}

#[test]
fn test_truncate() {
    type AS = ArrayString<U8, Pattern<0xEF>, 16>;
    let mut s = AS::try_from("2€").unwrap();

    s.truncate(1);
    assert_eq!(s, "2");
    check_spare_memory(&s, 0xEF);

    s.truncate(0);
    assert_eq!(s, "");
    check_spare_memory(&s, 0xEF);
}

#[test]
#[should_panic]
fn test_truncate_panics() {
    let mut s = array_str![16; "2€"];
    s.truncate(2);
}
