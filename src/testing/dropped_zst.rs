#![allow(dead_code)]
#![allow(unused_macros)]

use std::{
    any::TypeId,
    collections::HashMap,
    ptr,
    sync::{Mutex, Once},
};

#[macro_export]
macro_rules! gen_dropped_zst {
    ($name:ident) => {
        use $crate::testing::dropped_zst;

        #[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
        struct $name {}
        assert_eq!(core::mem::size_of::<$name>(), 0);

        impl $name {
            pub fn new() -> Self {
                dropped_zst::inc_new::<Self>();
                Self {}
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                dropped_zst::inc_drop::<Self>();
            }
        }

        impl Clone for $name {
            fn clone(&self) -> Self {
                dropped_zst::inc_clone::<Self>();
                Self {}
            }
        }
    };
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Counters {
    pub new: usize,
    pub clone: usize,
    pub drop: usize,
}

impl Counters {
    pub fn is_zero(&self) -> bool {
        self.new == 0 && self.clone == 0 && self.drop == 0
    }
}

type Table = HashMap<TypeId, Counters>;

static mut P_TABLE: *const Mutex<Table> = ptr::null();
static P_TABLE_ONCE: Once = Once::new();

fn get_table_mutex<'a>() -> &'a Mutex<Table> {
    P_TABLE_ONCE.call_once(|| unsafe {
        let b = Box::new(Mutex::new(Table::new()));
        P_TABLE = Box::into_raw(b);
    });
    unsafe { &*P_TABLE }
}

pub fn counters<T: 'static>() -> Counters {
    let mut t = get_table_mutex().lock().unwrap();
    #[allow(clippy::clone_on_copy)]
    t.entry(TypeId::of::<T>())
        .or_insert_with(Counters::default)
        .clone()
}

fn update_counters<T, F>(f: F)
where
    T: 'static,
    F: FnOnce(&mut Counters),
{
    let mut t = get_table_mutex().lock().unwrap();
    let counters = t.entry(TypeId::of::<T>()).or_insert_with(Counters::default);
    f(counters)
}

pub fn inc_new<T: 'static>() {
    update_counters::<T, _>(|c| c.new += 1);
}

pub fn inc_drop<T: 'static>() {
    update_counters::<T, _>(|c| c.drop += 1);
}

pub fn inc_clone<T: 'static>() {
    update_counters::<T, _>(|c| c.clone += 1);
}

#[cfg(all(test, feature = "std"))]
mod testing {
    use super::{counters, Counters};

    #[test]
    fn test_dropped_zst() {
        gen_dropped_zst!(T);
        assert!(counters::<T>().is_zero());

        let v = T::new();
        assert_eq!(
            counters::<T>(),
            Counters {
                new: 1,
                clone: 0,
                drop: 0
            }
        );

        let c = v.clone();
        assert_eq!(
            counters::<T>(),
            Counters {
                new: 1,
                clone: 1,
                drop: 0
            }
        );

        drop(v);
        assert_eq!(
            counters::<T>(),
            Counters {
                new: 1,
                clone: 1,
                drop: 1
            }
        );

        drop(c);
        assert_eq!(
            counters::<T>(),
            Counters {
                new: 1,
                clone: 1,
                drop: 2
            }
        );
    }
}
