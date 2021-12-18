use core::{
    cell::{Cell, RefCell},
    ops::RangeBounds,
};

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

    pub fn n_allocated(&self) -> usize {
        self.n_allocated.get()
    }

    pub fn dropped_range<R: RangeBounds<usize> + IntoIterator<Item = usize>>(
        &self,
        range: R,
    ) -> bool {
        let arr = self.arr.borrow();
        for (i, b) in arr.iter().enumerate() {
            if range.contains(&i) && !*b {
                return false;
            } else if !range.contains(&i) && *b {
                return false;
            }
        }
        for i in range {
            if i >= C || !arr[i] {
                return false;
            }
        }
        true
    }

    pub fn dropped_indices(&self, indices: &[usize]) -> bool {
        let arr = self.arr.borrow();
        for (i, b) in arr.iter().enumerate() {
            let is_found = indices.iter().find(|&x| *x == i).is_some();
            if is_found && !*b {
                return false;
            } else if !is_found && *b {
                return false;
            }
        }
        for i in indices {
            if *i >= C || !arr[*i] {
                return false;
            }
        }
        true
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
