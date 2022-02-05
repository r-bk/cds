use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::{Duration, Instant};

const CAP: usize = 8192;

macro_rules! bench_group {
    ($gsfx:ident, $gname:literal, $(($fname:literal, $f:expr)),*) => {
        fn $gsfx(c: &mut Criterion) {
            let mut group = c.benchmark_group($gname);

            $(
                group.bench_function($fname, $f);
            )*

            group.finish();
        }
    }
}

macro_rules! bf_push {
    ($vctor: expr) => {
        |b| {
            b.iter_custom(|mut iters| {
                let mut total = Duration::new(0, 0);
                let mut v = $vctor;
                while iters > 0 {
                    let tmp = (CAP as u64).min(iters);
                    v.clear();
                    let start = Instant::now();
                    for _i in 0..tmp {
                        let _ = v.push(0u64);
                    }
                    total += start.elapsed();
                    iters -= tmp;
                }
                total
            });
        }
    };
}

macro_rules! bf_pop {
    ($vctor: expr) => {
        |b| {
            b.iter_custom(|mut iters| {
                let mut total = Duration::new(0, 0);
                let mut v = $vctor;
                while iters > 0 {
                    let tmp = (CAP as u64).min(iters);
                    v.extend(0..tmp);
                    let start = Instant::now();
                    while !v.is_empty() {
                        v.pop();
                    }
                    total += start.elapsed();
                    iters -= tmp;
                }
                total
            });
        }
    };
}

macro_rules! bf_new {
    ($vctor: expr) => {
        |b| {
            b.iter(|| {
                let v = $vctor;
                black_box(v.len());
            });
        }
    };
}

macro_rules! bf_extend {
    ($vctor: expr) => {
        |b| {
            b.iter_custom(|mut iters| {
                let mut total = Duration::new(0, 0);
                let mut v = $vctor;
                while iters > 0 {
                    let tmp = (CAP as u64).min(iters);
                    v.clear();
                    let start = Instant::now();
                    v.extend(0..tmp);
                    total += start.elapsed();
                    iters -= tmp;
                }
                total
            });
        }
    };
}

macro_rules! bf_insert {
    ($vctor: expr) => {
        |b| {
            b.iter_custom(|mut iters| {
                let mut total = Duration::new(0, 0);
                let mut v = $vctor;
                while iters > 0 {
                    let tmp = (CAP as u64).min(iters);
                    v.clear();
                    let start = Instant::now();
                    for i in 0..tmp {
                        v.insert(i as usize, i);
                    }
                    total += start.elapsed();
                    iters -= tmp;
                }
                total
            });
        }
    };
}

macro_rules! bf_drain_no_consume {
    ($vctor: expr) => {
        |b| {
            b.iter_custom(|mut iters| {
                let mut total = Duration::new(0, 0);
                let mut v = $vctor;
                while iters > 0 {
                    let tmp = (CAP as u64).min(iters);
                    v.extend(0..tmp);
                    let start = Instant::now();
                    while !v.is_empty() {
                        v.drain(v.len() - 1..v.len());
                    }
                    total += start.elapsed();
                    iters -= tmp;
                    assert_eq!(v.len(), 0);
                }
                total
            });
        }
    };
}

macro_rules! bf_drain {
    ($vctor: expr) => {
        |b| {
            b.iter_custom(|mut iters| {
                let mut total = Duration::new(0, 0);
                let mut v = $vctor;
                while iters > 0 {
                    let tmp = (CAP as u64).min(iters);
                    v.extend(0..tmp);
                    let start = Instant::now();
                    while !v.is_empty() {
                        for e in v.drain(v.len() - 1..v.len()) {
                            black_box(e);
                        }
                    }
                    total += start.elapsed();
                    iters -= tmp;
                    assert_eq!(v.len(), 0);
                }
                total
            });
        }
    };
}

macro_rules! bf_remove {
    ($vctor: expr) => {
        |b| {
            b.iter_custom(|mut iters| {
                let mut total = Duration::new(0, 0);
                let mut v = $vctor;
                while iters > 0 {
                    let tmp = (CAP as u64).min(iters);
                    v.extend(0..tmp);
                    let start = Instant::now();
                    while !v.is_empty() {
                        v.remove(v.len() - 1);
                    }
                    total += start.elapsed();
                    iters -= tmp;
                    assert_eq!(v.len(), 0);
                }
                total
            });
        }
    };
}

macro_rules! bf_clear {
    ($vctor: expr) => {
        |b| {
            b.iter_custom(|mut iters| {
                let mut total = Duration::new(0, 0);
                let mut v = $vctor;
                while iters > 0 {
                    v.extend(0..1);
                    let start = Instant::now();
                    v.clear();
                    total += start.elapsed();
                    iters -= 1;
                    assert_eq!(v.len(), 0);
                }
                total
            });
        }
    };
}

macro_rules! bf_retain_no_remove {
    ($vctor: expr) => {
        |b| {
            b.iter_custom(|mut iters| {
                let mut total = Duration::new(0, 0);
                let mut v = $vctor;
                v.extend(0..1);
                while iters > 0 {
                    let start = Instant::now();
                    v.retain(|_e| true);
                    total += start.elapsed();
                    iters -= 1;
                    assert_eq!(v.len(), 1);
                }
                total
            });
        }
    };
}

macro_rules! bf_retain {
    ($vctor: expr) => {
        |b| {
            b.iter_custom(|mut iters| {
                let mut total = Duration::new(0, 0);
                let mut v = $vctor;
                while iters > 0 {
                    v.extend(0..1);
                    let start = Instant::now();
                    v.retain(|_e| false);
                    total += start.elapsed();
                    iters -= 1;
                    assert_eq!(v.len(), 0);
                }
                total
            });
        }
    };
}

macro_rules! bf_truncate {
    ($vctor: expr) => {
        |b| {
            b.iter_custom(|mut iters| {
                let mut total = Duration::new(0, 0);
                let mut v = $vctor;
                while iters > 0 {
                    let tmp = (CAP as u64).min(iters);
                    v.extend(0..tmp);
                    let start = Instant::now();
                    while !v.is_empty() {
                        v.truncate(v.len() - 1);
                    }
                    total += start.elapsed();
                    iters -= tmp;
                    assert_eq!(v.len(), 0);
                }
                total
            });
        }
    };
}

macro_rules! bf_swap_remove {
    ($vctor: expr) => {
        |b| {
            b.iter_custom(|mut iters| {
                let mut total = Duration::new(0, 0);
                let mut v = $vctor;
                while iters > 0 {
                    let tmp = (CAP as u64).min(iters);
                    v.clear();
                    v.extend(0..tmp);
                    let start = Instant::now();
                    while v.len() > 1 {
                        let e = v.swap_remove(v.len() - 1);
                        black_box(e);
                    }
                    total += start.elapsed();
                    iters -= tmp;
                }
                total
            });
        }
    };
}

macro_rules! bf_resize {
    ($vctor: expr) => {
        |b| {
            b.iter_custom(|mut iters| {
                let mut total = Duration::new(0, 0);
                let mut v = $vctor;
                while iters > 0 {
                    let tmp = (CAP as u64).min(iters);
                    v.clear();
                    let start = Instant::now();
                    while v.len() < CAP {
                        let _ = v.resize(v.len() + 1, 0);
                    }
                    total += start.elapsed();
                    iters -= tmp;
                    assert_eq!(v.len(), CAP);
                }
                total
            });
        }
    };
}

macro_rules! bf_resize_with {
    ($vctor: expr) => {
        |b| {
            b.iter_custom(|mut iters| {
                let mut total = Duration::new(0, 0);
                let mut v = $vctor;
                while iters > 0 {
                    let tmp = (CAP as u64).min(iters);
                    v.clear();
                    let start = Instant::now();
                    while v.len() < CAP {
                        let _ = v.resize_with(v.len() + 1, || 7);
                    }
                    total += start.elapsed();
                    iters -= tmp;
                    assert_eq!(v.len(), CAP);
                }
                total
            });
        }
    };
}

bench_group!(
    bench_push,
    "arrayvec/push",
    ("cds", bf_push!(cds::array_vec![CAP;])),
    ("std", bf_push!(Vec::with_capacity(CAP))),
    ("arrayvec", bf_push!(arrayvec::ArrayVec::<_, CAP>::new())),
    ("smallvec", bf_push!(smallvec::SmallVec::<[_; CAP]>::new())),
    ("heapless", bf_push!(heapless::Vec::<_, CAP>::new()))
);

bench_group!(
    bench_pop,
    "arrayvec/pop",
    ("cds", bf_pop!(cds::array_vec![CAP;])),
    ("std", bf_pop!(Vec::with_capacity(CAP))),
    ("arrayvec", bf_pop!(arrayvec::ArrayVec::<_, CAP>::new())),
    ("smallvec", bf_pop!(smallvec::SmallVec::<[_; CAP]>::new())),
    ("heapless", bf_pop!(heapless::Vec::<_, CAP>::new()))
);

bench_group!(
    bench_extend,
    "arrayvec/extend",
    ("cds", bf_extend!(cds::array_vec![CAP;])),
    ("std", bf_extend!(Vec::with_capacity(CAP))),
    ("arrayvec", bf_extend!(arrayvec::ArrayVec::<_, CAP>::new())),
    (
        "smallvec",
        bf_extend!(smallvec::SmallVec::<[_; CAP]>::new())
    ),
    ("heapless", bf_extend!(heapless::Vec::<_, CAP>::new()))
);

bench_group!(
    bench_insert,
    "arrayvec/insert",
    ("cds", bf_insert!(cds::array_vec![CAP;])),
    ("std", bf_insert!(Vec::with_capacity(CAP))),
    ("arrayvec", bf_insert!(arrayvec::ArrayVec::<_, CAP>::new())),
    (
        "smallvec",
        bf_insert!(smallvec::SmallVec::<[_; CAP]>::new())
    )
);

bench_group!(
    bench_new,
    "arrayvec/new",
    ("cds", bf_new!(cds::array_vec![CAP; u64])),
    ("arrayvec", bf_new!(arrayvec::ArrayVec::<u64, CAP>::new())),
    ("smallvec", bf_new!(smallvec::SmallVec::<[u64; CAP]>::new())),
    ("heapless", bf_new!(heapless::Vec::<u64, CAP>::new()))
);

bench_group!(
    bench_drain_no_consume,
    "arrayvec/drain_no_consume",
    ("cds", bf_drain_no_consume!(cds::array_vec![CAP;])),
    ("std", bf_drain_no_consume!(Vec::with_capacity(CAP))),
    (
        "arrayvec",
        bf_drain_no_consume!(arrayvec::ArrayVec::<_, CAP>::new())
    ),
    (
        "smallvec",
        bf_drain_no_consume!(smallvec::SmallVec::<[_; CAP]>::new())
    )
);

bench_group!(
    bench_drain,
    "arrayvec/drain",
    ("cds", bf_drain!(cds::array_vec![CAP;])),
    ("std", bf_drain!(Vec::with_capacity(CAP))),
    ("arrayvec", bf_drain!(arrayvec::ArrayVec::<_, CAP>::new())),
    ("smallvec", bf_drain!(smallvec::SmallVec::<[_; CAP]>::new()))
);

bench_group!(
    bench_remove,
    "arrayvec/remove",
    ("cds", bf_remove!(cds::array_vec![CAP;])),
    ("std", bf_remove!(Vec::with_capacity(CAP))),
    ("arrayvec", bf_remove!(arrayvec::ArrayVec::<_, CAP>::new())),
    (
        "smallvec",
        bf_remove!(smallvec::SmallVec::<[_; CAP]>::new())
    )
);

bench_group!(
    bench_clear,
    "arrayvec/clear",
    ("cds", bf_clear!(cds::array_vec![CAP;])),
    ("std", bf_clear!(Vec::with_capacity(CAP))),
    ("arrayvec", bf_clear!(arrayvec::ArrayVec::<_, CAP>::new())),
    ("smallvec", bf_clear!(smallvec::SmallVec::<[_; CAP]>::new())),
    ("heapless", bf_clear!(heapless::Vec::<u64, CAP>::new()))
);

bench_group!(
    bench_retain_no_remove,
    "arrayvec/retain_no_remove",
    ("cds", bf_retain_no_remove!(cds::array_vec![CAP;])),
    ("std", bf_retain_no_remove!(Vec::with_capacity(CAP))),
    (
        "arrayvec",
        bf_retain_no_remove!(arrayvec::ArrayVec::<_, CAP>::new())
    ),
    (
        "smallvec",
        bf_retain_no_remove!(smallvec::SmallVec::<[_; CAP]>::new())
    )
);

bench_group!(
    bench_retain,
    "arrayvec/retain",
    ("cds", bf_retain!(cds::array_vec![CAP;])),
    ("std", bf_retain!(Vec::with_capacity(CAP))),
    ("arrayvec", bf_retain!(arrayvec::ArrayVec::<_, CAP>::new())),
    (
        "smallvec",
        bf_retain!(smallvec::SmallVec::<[_; CAP]>::new())
    )
);

bench_group!(
    bench_truncate,
    "arrayvec/truncate",
    ("cds", bf_truncate!(cds::array_vec![CAP;])),
    ("std", bf_truncate!(Vec::with_capacity(CAP))),
    (
        "arrayvec",
        bf_truncate!(arrayvec::ArrayVec::<_, CAP>::new())
    ),
    (
        "smallvec",
        bf_truncate!(smallvec::SmallVec::<[_; CAP]>::new())
    ),
    ("heapless", bf_truncate!(heapless::Vec::<_, CAP>::new()))
);

bench_group!(
    bench_swap_remove,
    "arrayvec/swap_remove",
    ("cds", bf_swap_remove!(cds::array_vec![CAP;])),
    ("std", bf_swap_remove!(Vec::with_capacity(CAP))),
    (
        "arrayvec",
        bf_swap_remove!(arrayvec::ArrayVec::<_, CAP>::new())
    ),
    (
        "smallvec",
        bf_swap_remove!(smallvec::SmallVec::<[_; CAP]>::new())
    ),
    ("heapless", bf_swap_remove!(heapless::Vec::<_, CAP>::new()))
);

bench_group!(
    bench_resize,
    "arrayvec/resize",
    ("cds", bf_resize!(cds::array_vec![CAP;])),
    ("std", bf_resize!(Vec::with_capacity(CAP))),
    (
        "smallvec",
        bf_resize!(smallvec::SmallVec::<[_; CAP]>::new())
    ),
    ("heapless", bf_resize!(heapless::Vec::<_, CAP>::new()))
);

bench_group!(
    bench_resize_with,
    "arrayvec/resize_with",
    ("cds", bf_resize_with!(cds::array_vec![CAP;])),
    ("std", bf_resize_with!(Vec::with_capacity(CAP))),
    (
        "smallvec",
        bf_resize_with!(smallvec::SmallVec::<[_; CAP]>::new())
    )
);

criterion_group!(
    benches,
    bench_push,
    bench_pop,
    bench_new,
    bench_extend,
    bench_insert,
    bench_drain_no_consume,
    bench_drain,
    bench_remove,
    bench_clear,
    bench_retain_no_remove,
    bench_retain,
    bench_truncate,
    bench_swap_remove,
    bench_resize,
    bench_resize_with,
);
criterion_main!(benches);
