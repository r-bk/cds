use core::cell::{Cell, RefCell};

pub struct Track<const C: usize> {
    arr: RefCell<[bool; C]>,
    idx: Cell<usize>,
    n_allocated: Cell<usize>,
}

pub struct TrackIter<'a, const C: usize> {
    track: &'a Track<C>,
    count: usize,
}

pub struct Dropped<'a, const C: usize> {
    track: &'a Track<C>,
    idx: usize,
}

#[allow(dead_code)]
impl<const C: usize> Track<C> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn alloc(&self) -> Dropped<'_, C> {
        let idx = self.idx.get();
        assert!(idx < C);
        self.idx.set(idx + 1);
        self.n_allocated.set(self.n_allocated.get() + 1);
        Dropped { track: self, idx }
    }

    pub fn take(&self, count: usize) -> TrackIter<'_, C> {
        TrackIter { track: self, count }
    }

    fn free(&self, d: &Dropped<'_, C>) {
        assert_eq!(self.get(d.idx), false);
        self.n_allocated.set(self.n_allocated.get() - 1);
        self.set(d.idx, true);
    }

    pub fn get(&self, index: usize) -> bool {
        let arr = self.arr.borrow();
        arr[index]
    }

    fn set(&self, index: usize, value: bool) {
        let mut arr = self.arr.borrow_mut();
        arr[index] = value;
    }

    pub fn clear(&self) {
        self.n_allocated.set(0);
        self.idx.set(0);
        let mut arr = self.arr.borrow_mut();
        arr.fill(false);
    }

    pub fn dropped(&self) -> heapless::Vec<usize, C> {
        let mut tmp = heapless::Vec::new();
        let arr = self.arr.borrow();
        for (i, b) in arr.iter().enumerate() {
            if *b {
                tmp.push(i).expect("push failed");
            }
        }
        tmp
    }
}

impl<'a, const C: usize> Iterator for TrackIter<'a, C> {
    type Item = Dropped<'a, C>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count > 0 {
            self.count -= 1;
            Some(self.track.alloc())
        } else {
            None
        }
    }
}

impl<const C: usize> Default for Track<C> {
    fn default() -> Self {
        Track {
            arr: RefCell::new([false; C]),
            idx: Cell::new(0),
            n_allocated: Cell::new(0),
        }
    }
}

impl<const C: usize> Drop for Dropped<'_, C> {
    fn drop(&mut self) {
        self.track.free(self);
    }
}

impl<const C: usize> Clone for Dropped<'_, C> {
    fn clone(&self) -> Self {
        self.track.alloc()
    }
}
