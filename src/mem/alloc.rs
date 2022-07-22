use crate::mem::{errors::ReservationError, SpareMemoryPolicy};
use ::alloc::alloc::{self, Layout};
use core::{mem, ptr};

pub const DOHAE: bool = true; // call `handle_allocation_error`
pub const NOHAE: bool = false; // do not call `handle_allocation_error`

#[inline]
pub fn alloc_buffer<T, const HAE: bool>(cap: usize) -> Result<*mut T, ReservationError> {
    unsafe {
        let new_layout = Layout::array::<T>(cap).map_err(|_| ReservationError::CapacityOverflow)?;
        if new_layout.size() > isize::MAX as usize {
            return Err(ReservationError::CapacityOverflow);
        }

        let tmp: *mut T = alloc::alloc(new_layout).cast();
        if tmp.is_null() {
            if HAE {
                alloc::handle_alloc_error(new_layout);
            }
            return Err(ReservationError::AllocError { layout: new_layout });
        }

        Ok(tmp)
    }
}

/// Allocates a new buffer for `[T; cap]`.
///
/// # Safety
///
/// 1. `cap > 0`
/// 2. `size_of::<T>() != 0`
#[inline]
pub unsafe fn alloc_buffer_hae<T>(cap: usize) -> *mut T {
    debug_assert!(mem::size_of::<T>() != 0);
    debug_assert!(cap > 0);

    let new_layout = Layout::array::<T>(cap).unwrap();
    debug_assert!(new_layout.size() != 0);

    let tmp: *mut T = alloc::alloc(new_layout).cast();
    if tmp.is_null() {
        alloc::handle_alloc_error(new_layout);
    }
    tmp
}

#[inline]
pub fn realloc_buffer<T, SM: SpareMemoryPolicy<T>, const HAE: bool>(
    p: *mut T,
    old_len: usize,
    old_cap: usize,
    new_cap: usize,
) -> Result<*mut T, ReservationError> {
    debug_assert!(new_cap > old_cap);

    let old_layout = unsafe {
        Layout::from_size_align_unchecked(mem::size_of::<T>() * old_cap, mem::align_of::<T>())
    };

    let new_layout = Layout::array::<T>(new_cap).map_err(|_| ReservationError::CapacityOverflow)?;

    if new_layout.size() > isize::MAX as usize {
        return Err(ReservationError::CapacityOverflow);
    }

    unsafe {
        if SM::NOOP {
            let tmp: *mut T = alloc::realloc(p.cast(), old_layout, new_layout.size()).cast();
            if tmp.is_null() {
                if HAE {
                    alloc::handle_alloc_error(new_layout);
                }
                return Err(ReservationError::AllocError { layout: new_layout });
            }
            Ok(tmp)
        } else {
            let tmp: *mut T = alloc::alloc(new_layout).cast();
            if tmp.is_null() {
                if HAE {
                    alloc::handle_alloc_error(new_layout);
                }
                return Err(ReservationError::AllocError { layout: new_layout });
            }
            // copy the old buffer including its spare memory
            ptr::copy_nonoverlapping(p, tmp, old_cap);
            SM::init(p, old_len);
            alloc::dealloc(p.cast(), old_layout);
            Ok(tmp)
        }
    }
}

/// Reallocates a buffer allocated by this module.
///
/// # Safety
///
/// 1. `p` is a memory buffer allocated by `alloc_buffer_hae`.
/// 2. `old_cap` is the capacity of `p` in elements
/// 3. `new_cap > old_cap` (and consequently not zero)
#[inline]
pub unsafe fn realloc_buffer_hae<T, SM>(
    p: *mut T,
    old_len: usize,
    old_cap: usize,
    new_cap: usize,
) -> *mut T
where
    SM: SpareMemoryPolicy<T>,
{
    debug_assert!(mem::size_of::<T>() != 0);
    debug_assert!(new_cap > old_cap);

    let new_layout = Layout::array::<T>(new_cap).unwrap();
    debug_assert!(new_layout.size() != 0);

    let old_layout =
        Layout::from_size_align_unchecked(mem::size_of::<T>() * old_cap, mem::align_of::<T>());

    if SM::NOOP {
        let tmp: *mut T = alloc::realloc(p.cast(), old_layout, new_layout.size()).cast();
        if tmp.is_null() {
            alloc::handle_alloc_error(new_layout);
        }
        tmp
    } else {
        let tmp: *mut T = alloc::alloc(new_layout).cast();
        if tmp.is_null() {
            alloc::handle_alloc_error(new_layout);
        }
        // copy the old buffer including its spare memory
        ptr::copy_nonoverlapping(p, tmp, old_cap);
        SM::init(p, old_len);
        alloc::dealloc(p.cast(), old_layout);
        tmp
    }
}

#[inline]
pub fn dealloc_buffer<T>(p: *mut T, cap: usize) {
    unsafe {
        let layout =
            Layout::from_size_align_unchecked(mem::size_of::<T>() * cap, mem::align_of::<T>());
        alloc::dealloc(p.cast(), layout);
    }
}
