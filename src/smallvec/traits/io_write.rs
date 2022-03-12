use crate::{
    len::LengthType,
    mem::SpareMemoryPolicy,
    smallvec::{SmallVec, NOHAE},
};

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl<L, SM, const C: usize> std::io::Write for SmallVec<u8, C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.try_copy_from_slice_impl::<NOHAE>(buf)
            .map_err(|_| std::io::Error::from(std::io::ErrorKind::OutOfMemory))?;
        Ok(buf.len())
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{len::U8, smallvec::SmallVec};
    use std::io::Write;

    #[test]
    fn test_io_write() {
        type SV = SmallVec<u8, 16>;
        let mut v = SV::new();
        assert_eq!(v.write(b"thisisatest").unwrap(), 11);
        assert_eq!(v, b"thisisatest");
        assert_eq!(v.write(b"ofiowrite").unwrap(), 9);
        assert_eq!(v, b"thisisatestofiowrite");
    }

    #[test]
    fn test_io_write_out_of_memory() {
        type SV = SmallVec<u8, 16, U8>;
        let mut v = SV::new();
        for _ in 0..255 {
            assert_eq!(v.write(b"a").unwrap(), 1);
        }
        assert_eq!(v.len(), 255);
        assert!(matches!(v.write(b"a"), Err(e) if e.kind() == std::io::ErrorKind::OutOfMemory));
        assert_eq!(v.len(), 255);
    }

    #[test]
    fn test_io_write_empty() {
        type SV = SmallVec<u8, 16>;
        let mut v = SV::new();
        assert_eq!(v.write(b"").unwrap(), 0);
        assert_eq!(v, []);
    }

    #[test]
    fn test_io_write_local() {
        type SV = SmallVec<u8, 16>;
        let mut v = SV::new();
        assert_eq!(v.write(b"abcdefghijklmnop").unwrap(), 16);
        assert_eq!(&v, b"abcdefghijklmnop");
    }

    #[test]
    fn test_io_write_flush() {
        type SV = SmallVec<u8, 5>;
        let mut v = SV::new();
        assert!(v.flush().is_ok());
        assert_eq!(v.write(b"write").unwrap(), 5);
        assert!(v.flush().is_ok());
    }
}
