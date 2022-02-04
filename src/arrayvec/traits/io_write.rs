use crate::{
    arrayvec::ArrayVec,
    defs::{LengthType, SpareMemoryPolicy},
};
use core::slice;

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl<L, SM, const C: usize> std::io::Write for ArrayVec<u8, L, SM, C>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let len = self.spare_capacity().min(buf.len());
        unsafe { self.copy_from_slice_unchecked(slice::from_raw_parts(buf.as_ptr(), len)) };
        Ok(len)
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        arrayvec::ArrayVec,
        defs::{Uninitialized, Usize},
    };
    use std::io::Write;

    #[test]
    fn test_io_write() {
        type A = ArrayVec<u8, Usize, Uninitialized, 16>;
        let mut av = A::new();
        assert_eq!(av.write(b"thisisatest").unwrap(), 11);
        assert_eq!(av, b"thisisatest");
        assert_eq!(av.write(b"ofiowrite").unwrap(), 5);
        assert_eq!(av, b"thisisatestofiow");
    }

    #[test]
    fn test_io_write_empty() {
        type A = ArrayVec<u8, Usize, Uninitialized, 16>;
        let mut av = A::new();
        assert_eq!(av.write(b"").unwrap(), 0);
        assert_eq!(av, []);
    }

    #[test]
    fn test_io_write_full() {
        type A = ArrayVec<u8, Usize, Uninitialized, 5>;
        let mut av = A::new();
        assert_eq!(av.write(b"write").unwrap(), 5);
        assert_eq!(av, b"write");
    }

    #[test]
    fn test_io_write_flush() {
        type A = ArrayVec<u8, Usize, Uninitialized, 5>;
        let mut av = A::new();
        assert!(av.flush().is_ok());
        assert_eq!(av.write(b"write").unwrap(), 5);
        assert!(av.flush().is_ok());
    }
}
