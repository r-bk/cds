//! A string with "small size" optimization.

use crate::{
    len::{LengthType, Usize},
    mem::{SpareMemoryPolicy, Uninitialized},
};

mod buffer;

pub struct SmallString<const C: usize, L = Usize, SM = Uninitialized>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    /// The buffer
    buf: buffer::Buffer<C, L, SM>,

    /// The length of small string when local; the capacity of the buffer when on heap
    capacity: L,
}
