use crate::{arraystring::ArrayString, len::LengthType, mem::SpareMemoryPolicy};
use core::{clone::Clone, ptr};

impl<L, SM, const C: usize> Clone for ArrayString<L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn clone(&self) -> Self {
        let len = self.len();
        let mut s = Self::new_raw(len);
        unsafe {
            ptr::copy_nonoverlapping(self.as_ptr(), s.as_mut_ptr(), len);
            SM::init(s.as_mut_ptr().add(len), C - len);
        }
        s
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        let old_len = self.len();
        let len = source.len();
        unsafe {
            ptr::copy_nonoverlapping(source.as_ptr(), self.as_mut_ptr(), len);
            self.set_len(len);
        }

        let spare_bytes = if old_len > len { old_len - len } else { 0 };
        unsafe {
            SM::init(self.as_mut_ptr().add(len), spare_bytes);
        }
    }
}

#[cfg(test)]
mod testing {
    use crate as cds;
    use cds::{
        arraystring::{test_arraystring::check_spare_memory, ArrayString},
        len::U8,
        mem::Pattern,
    };

    const PATTERN: u8 = 0xBC;
    type AS = ArrayString<U8, Pattern<PATTERN>, 16>;

    #[test]
    fn test_clone() {
        let s = AS::try_from("cds").unwrap();
        let d = s.clone();
        assert_eq!(d, "cds");
        check_spare_memory(&s, PATTERN);
    }

    #[test]
    fn test_clone_from() {
        let s = AS::try_from("cds").unwrap();
        let s2 = AS::try_from("cdscdscds").unwrap();

        let mut d = AS::try_from("onetwo").unwrap();
        assert_eq!(d, "onetwo");
        check_spare_memory(&d, PATTERN);

        d.clone_from(&s);
        assert_eq!(d, "cds");
        check_spare_memory(&d, PATTERN);

        d.clone_from(&s2);
        assert_eq!(d, "cdscdscds");
        check_spare_memory(&d, PATTERN);
    }
}
