use crate::{len::LengthType, mem::SpareMemoryPolicy};
use core::{
    clone::Clone,
    marker::{Copy, PhantomData},
    mem::MaybeUninit,
    ptr,
};

pub struct Local<const C: usize, SM>
where
    SM: SpareMemoryPolicy<u8>,
{
    pub arr: [MaybeUninit<u8>; C],
    phantom: PhantomData<SM>,
}

impl<const C: usize, SM> Clone for Local<C, SM>
where
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<const C: usize, SM> Copy for Local<C, SM> where SM: SpareMemoryPolicy<u8> {}

impl<const C: usize, SM> Local<C, SM>
where
    SM: SpareMemoryPolicy<u8>,
{
    #[inline(always)]
    pub fn new() -> Self {
        let mut local = Self {
            arr: unsafe { MaybeUninit::uninit().assume_init() },
            phantom: PhantomData,
        };
        unsafe {
            SM::init(local.arr.as_mut_ptr() as *mut u8, C);
        }
        local
    }

    #[inline(always)]
    pub unsafe fn from_bytes(bytes: &[u8]) -> Self {
        let mut local = Self {
            arr: MaybeUninit::uninit().assume_init(),
            phantom: PhantomData,
        };
        let len = bytes.len();
        let tgt = local.arr.as_mut_ptr() as *mut u8;
        ptr::copy_nonoverlapping(bytes.as_ptr(), tgt, len);
        SM::init(tgt.add(len), C - len);
        local
    }
}

pub union Buffer<const C: usize, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    heap: (*mut u8, L),
    local: Local<C, SM>,
}

impl<const C: usize, L, SM> Buffer<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    pub fn new() -> Self {
        Self {
            local: Local::<C, SM>::new(),
        }
    }

    #[inline]
    pub unsafe fn local_from_bytes(bytes: &[u8]) -> Self {
        Self {
            local: Local::<C, SM>::from_bytes(bytes),
        }
    }

    #[inline]
    pub unsafe fn heap(p: *mut u8, len: usize) -> Self {
        if SM::NOOP {
            Self {
                heap: (p, L::new(len)),
            }
        } else {
            let mut tmp = Self::new();
            tmp.heap.0 = p;
            tmp.heap.1 = L::new(len);
            tmp
        }
    }
}

impl<const C: usize, L, SM> Buffer<C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<u8>,
{
    #[inline]
    pub fn local_ptr(&self) -> *const u8 {
        unsafe { self.local.arr.as_ptr() as *const u8 }
    }

    #[inline]
    pub fn local_mut_ptr(&mut self) -> *mut u8 {
        unsafe { self.local.arr.as_mut_ptr() as *mut u8 }
    }

    #[inline]
    pub fn heap_mut_ptr(&mut self) -> *mut u8 {
        unsafe { self.heap.0 }
    }

    #[inline]
    pub fn heap_len_p(&self) -> (L, *const u8) {
        unsafe { (self.heap.1, self.heap.0) }
    }

    #[inline]
    pub fn heap_len_mut_p(&self) -> (L, *mut u8) {
        unsafe { (self.heap.1, self.heap.0) }
    }

    #[inline]
    pub fn heap_len(&self) -> L {
        unsafe { self.heap.1 }
    }

    #[inline]
    pub fn set_heap_len(&mut self, l: L) {
        self.heap.1 = l;
    }

    #[inline]
    pub fn set_heap(&mut self, p: *mut u8, l: L) {
        self.heap.0 = p;
        self.heap.1 = l;
    }

    #[inline]
    pub fn set_heap_ptr(&mut self, p: *mut u8) {
        self.heap.0 = p;
    }

    #[inline]
    pub fn heap_mut_len_mut_p(&mut self) -> (&mut L, *mut u8) {
        unsafe { (&mut self.heap.1, self.heap.0) }
    }
}
