use crate::{len::LengthType, mem::SpareMemoryPolicy, smallvec::SmallVec};
use core::{
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
};

pub struct Local<T, const C: usize, SM>
where
    SM: SpareMemoryPolicy<T>,
{
    pub arr: [MaybeUninit<T>; C],
    phantom: PhantomData<SM>,
}

impl<T, const C: usize, SM> Local<T, C, SM>
where
    SM: SpareMemoryPolicy<T>,
{
    #[inline(always)]
    fn new() -> Self {
        let mut local = Self {
            arr: unsafe { MaybeUninit::uninit().assume_init() },
            phantom: PhantomData,
        };
        unsafe {
            SM::init(local.arr.as_mut_ptr() as *mut T, C);
        }
        local
    }
}

pub union Buffer<T, const C: usize, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    local: ManuallyDrop<Local<T, C, SM>>,
    heap: (*mut T, L),
}

impl<T, const C: usize, L, SM> Buffer<T, C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            local: ManuallyDrop::new(Local::new()),
        }
    }

    #[inline]
    pub fn heap_len(&self) -> L {
        unsafe { self.heap.1 }
    }

    #[inline]
    pub fn heap_ptr(&self) -> *const T {
        unsafe { self.heap.0 }
    }

    #[inline]
    pub fn heap_mut_ptr(&mut self) -> *mut T {
        unsafe { self.heap.0 }
    }

    #[inline]
    pub fn set_heap(&mut self, p: *mut T, l: L) {
        self.heap.0 = p;
        self.heap.1 = l;
    }

    #[inline]
    pub fn set_heap_ptr(&mut self, p: *mut T) {
        self.heap.0 = p;
    }

    #[inline]
    pub fn set_heap_len(&mut self, l: L) {
        self.heap.1 = l;
    }

    #[inline]
    pub fn heap_len_add_assign(&mut self, c: usize) {
        unsafe { self.heap.1.add_assign(c) }
    }

    #[inline]
    pub fn local_ptr(&self) -> *const T {
        unsafe { (*self.local).arr.as_ptr() as *const T }
    }

    #[inline]
    pub fn local_mut_ptr(&mut self) -> *mut T {
        unsafe { (*self.local).arr.as_mut_ptr() as *mut T }
    }

    #[inline]
    pub fn heap_len_p(&self) -> (L, *const T) {
        unsafe { (self.heap.1, self.heap.0) }
    }

    #[inline]
    pub fn heap_len_mut_p(&mut self) -> (L, *mut T) {
        unsafe { (self.heap.1, self.heap.0) }
    }

    #[inline]
    pub fn heap_mut_len_mut_p(&mut self) -> (&mut L, *mut T) {
        unsafe { (&mut self.heap.1, self.heap.0) }
    }
}

#[derive(Debug)]
pub struct SetLenOnDrop<'a, T, const C: usize, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    pub len: usize,
    pub sv: &'a mut SmallVec<T, C, L, SM>,
    pub armed: bool,
}

impl<'a, T, const C: usize, L, SM> SetLenOnDrop<'a, T, C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    pub fn new(sv: &'a mut SmallVec<T, C, L, SM>, len: usize) -> Self {
        Self {
            len,
            sv,
            armed: true,
        }
    }

    #[inline]
    pub fn unarmed(sv: &'a mut SmallVec<T, C, L, SM>, len: usize) -> Self {
        Self {
            len,
            sv,
            armed: false,
        }
    }
}

impl<'a, T, const C: usize, L, SM> Drop for SetLenOnDrop<'a, T, C, L, SM>
where
    L: LengthType,
    SM: SpareMemoryPolicy<T>,
{
    #[inline]
    fn drop(&mut self) {
        if self.armed {
            unsafe { self.sv.set_len(self.len) }
        }
    }
}
