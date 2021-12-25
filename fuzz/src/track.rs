use std::{
    cell::RefCell,
    collections::VecDeque,
    ops::{Bound, RangeBounds},
};

#[derive(Debug)]
pub struct Element<'a, const C: usize> {
    track: &'a Track<C>,
    idx: usize,
}

impl<'a, const C: usize> Element<'a, C> {
    pub fn idx(&self) -> usize {
        self.idx
    }
}

#[derive(Debug)]
pub struct Track<const C: usize> {
    slots: RefCell<[bool; C]>,
    free: RefCell<VecDeque<usize>>,
}

struct Take<'a, const C: usize> {
    track: &'a Track<C>,
    count: usize,
}

impl<const C: usize> Track<C> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn alloc(&self) -> Element<'_, C> {
        let mut free = self.free.borrow_mut();
        let mut slots = self.slots.borrow_mut();

        let idx = free.pop_front().expect("no free slots");
        assert!(!slots[idx]);
        slots[idx] = true;

        Element::<C> { track: self, idx }
    }

    pub fn free(&self, e: &Element<'_, C>) {
        assert_eq!(self as *const Self, e.track as *const Self);

        let mut slots = self.slots.borrow_mut();
        assert!(slots[e.idx]);
        slots[e.idx] = false;

        let mut free = self.free.borrow_mut();
        free.push_back(e.idx);
    }

    pub fn take(&self, count: usize) -> impl Iterator<Item = Element<'_, C>> {
        Take { track: self, count }
    }

    pub fn is_allocated(&self, indices: &[usize]) -> bool {
        let slots = self.slots.borrow();
        for i in indices.iter() {
            if !slots[*i] {
                return false;
            }
        }
        true
    }

    pub fn is_allocated_exact(&self, indices: &[usize]) -> bool {
        let slots = self.slots.borrow();
        for (i, s) in slots.iter().enumerate() {
            let expected = indices.contains(&i);
            if *s != expected {
                return false;
            }
        }
        true
    }

    pub fn is_allocated_range<R: RangeBounds<usize>>(&self, r: R) -> bool {
        let start = match r.start_bound() {
            Bound::Unbounded => 0,
            Bound::Included(s) => *s,
            Bound::Excluded(s) => *s + 1,
        };

        let end = match r.end_bound() {
            Bound::Unbounded => C,
            Bound::Included(e) => (*e).checked_add(1).unwrap(),
            Bound::Excluded(e) => *e,
        };

        let slots = self.slots.borrow();
        for i in start..end {
            if !slots[i] {
                return false;
            }
        }
        true
    }

    pub fn is_allocated_range_exact<R: RangeBounds<usize>>(&self, r: R) -> bool {
        let slots = self.slots.borrow();
        for (i, s) in slots.iter().enumerate() {
            let expected = r.contains(&i);
            if *s != expected {
                return false;
            }
        }
        true
    }
}

impl<const C: usize> Default for Track<C> {
    fn default() -> Self {
        Self {
            slots: RefCell::new([false; C]),
            free: RefCell::new(VecDeque::from_iter(0..C)),
        }
    }
}

impl<const C: usize> PartialEq<[usize]> for Track<C> {
    fn eq(&self, other: &[usize]) -> bool {
        self.is_allocated_exact(other)
    }
}

impl<const C: usize> PartialEq<&[usize]> for Track<C> {
    fn eq(&self, other: &&[usize]) -> bool {
        self.is_allocated_exact(other)
    }
}

impl<'a, const C: usize> Drop for Element<'a, C> {
    fn drop(&mut self) {
        self.track.free(self)
    }
}

impl<'a, const C: usize> Iterator for Take<'a, C> {
    type Item = Element<'a, C>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count > 0 {
            self.count -= 1;
            Some(self.track.alloc())
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.count))
    }
}
