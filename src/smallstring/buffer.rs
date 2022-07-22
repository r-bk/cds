use crate::{len::LengthType, mem::SpareMemoryPolicy};
use core::mem::{ManuallyDrop, MaybeUninit};

pub struct Local<const C: usize, SM>
where
    SM: SpareMemoryPolicy<u8>,
{
    pub arr: [MaybeUninit<u8>; C],
}

pub union Buffer<const C: usize, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    heap: (*mut u8, L),
    local: ManuallyDrop<Local<C, SM>>,
}
