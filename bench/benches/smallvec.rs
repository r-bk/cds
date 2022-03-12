use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::{Duration, Instant};

const MAX_CAP: usize = 8192;
/// local cap for heap benchmarks
const HEAP_CAP: usize = 2;

type HSV = cds::smallvec::SmallVec<u64, HEAP_CAP>;
type HTV = tinyvec::TinyVec<[u64; HEAP_CAP]>;

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
                    let tmp = (MAX_CAP as u64).min(iters);
                    v.clear();
                    let start = Instant::now();
                    for _i in 0..tmp {
                        v.push(0u64);
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
                    let tmp = (MAX_CAP as u64).min(iters);
                    for _ in 0..tmp {
                        v.push(0);
                    }
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

macro_rules! bf_insert {
    ($vctor: expr) => {
        |b| {
            b.iter_custom(|mut iters| {
                let mut total = Duration::new(0, 0);
                let mut v = $vctor;
                while iters > 0 {
                    let tmp = (MAX_CAP as u64).min(iters);
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

macro_rules! bf_remove {
    ($vctor: expr) => {
        |b| {
            b.iter_custom(|mut iters| {
                let mut total = Duration::new(0, 0);
                let mut v = $vctor;
                while iters > 0 {
                    let tmp = (MAX_CAP as u64).min(iters);
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

macro_rules! bf_swap_remove {
    ($vctor: expr) => {
        |b| {
            b.iter_custom(|mut iters| {
                let mut total = Duration::new(0, 0);
                let mut v = $vctor;
                while iters > 0 {
                    let tmp = (MAX_CAP as u64).min(iters);
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
                    let tmp = (MAX_CAP as u64).min(iters);
                    v.clear();
                    let start = Instant::now();
                    for i in 0..tmp {
                        let _ = v.resize((i + 1) as usize, 0);
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
                    let tmp = (MAX_CAP as u64).min(iters);
                    v.extend(0..tmp);
                    let mut len = v.len();
                    let start = Instant::now();
                    while len > 0 {
                        v.drain(len - 1..len);
                        len -= 1;
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
                    let tmp = (MAX_CAP as u64).min(iters);
                    v.extend(0..tmp);
                    let mut len = v.len();
                    let start = Instant::now();
                    while len > 0 {
                        for e in v.drain(len - 1..len) {
                            black_box(e);
                            len -= 1;
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

bench_group!(
    bench_push_heap,
    "smallvec/push/heap",
    ("cds", bf_push!(HSV::with_capacity(MAX_CAP))),
    ("std", bf_push!(Vec::with_capacity(MAX_CAP))),
    ("tinyvec", bf_push!(HTV::with_capacity(MAX_CAP))),
    (
        "smallvec",
        bf_push!(smallvec::SmallVec::<[_; HEAP_CAP]>::with_capacity(MAX_CAP))
    )
);

bench_group!(
    bench_push_local,
    "smallvec/push/local",
    ("cds", bf_push!(cds::small_vec![MAX_CAP;])),
    ("std", bf_push!(Vec::with_capacity(MAX_CAP))),
    ("tinyvec", bf_push!(tinyvec::tiny_vec!([_; MAX_CAP]))),
    (
        "smallvec",
        bf_push!(smallvec::SmallVec::<[_; MAX_CAP]>::new())
    )
);

bench_group!(
    bench_pop_heap,
    "smallvec/pop/heap",
    ("cds", bf_pop!(HSV::with_capacity(MAX_CAP))),
    ("std", bf_pop!(Vec::with_capacity(MAX_CAP))),
    ("tinyvec", bf_pop!(HTV::with_capacity(MAX_CAP))),
    (
        "smallvec",
        bf_pop!(smallvec::SmallVec::<[_; HEAP_CAP]>::with_capacity(MAX_CAP))
    )
);

bench_group!(
    bench_pop_local,
    "smallvec/pop/local",
    ("cds", bf_pop!(cds::small_vec![MAX_CAP;])),
    ("std", bf_pop!(Vec::new())),
    ("tinyvec", bf_pop!(tinyvec::tiny_vec!([_; MAX_CAP]))),
    (
        "smallvec",
        bf_pop!(smallvec::SmallVec::<[_; MAX_CAP]>::new())
    )
);

bench_group!(
    bench_insert_heap,
    "smallvec/insert/heap",
    ("cds", bf_insert!(HSV::with_capacity(MAX_CAP))),
    ("std", bf_insert!(Vec::with_capacity(MAX_CAP))),
    ("tinyvec", bf_insert!(HTV::with_capacity(MAX_CAP))),
    (
        "smallvec",
        bf_insert!(smallvec::SmallVec::<[_; HEAP_CAP]>::with_capacity(MAX_CAP))
    )
);

bench_group!(
    bench_insert_local,
    "smallvec/insert/local",
    ("cds", bf_insert!(cds::small_vec![MAX_CAP;])),
    ("std", bf_insert!(Vec::new())),
    ("tinyvec", bf_insert!(tinyvec::tiny_vec!([_; MAX_CAP]))),
    (
        "smallvec",
        bf_insert!(smallvec::SmallVec::<[_; MAX_CAP]>::new())
    )
);

bench_group!(
    bench_remove_heap,
    "smallvec/remove/heap",
    ("cds", bf_remove!(HSV::with_capacity(MAX_CAP))),
    ("std", bf_remove!(Vec::with_capacity(MAX_CAP))),
    ("tinyvec", bf_remove!(HTV::with_capacity(MAX_CAP))),
    (
        "smallvec",
        bf_remove!(smallvec::SmallVec::<[_; HEAP_CAP]>::with_capacity(MAX_CAP))
    )
);

bench_group!(
    bench_remove_local,
    "smallvec/remove/local",
    ("cds", bf_remove!(cds::small_vec![MAX_CAP;])),
    ("std", bf_remove!(Vec::new())),
    ("tinyvec", bf_remove!(tinyvec::tiny_vec!([_; MAX_CAP]))),
    (
        "smallvec",
        bf_remove!(smallvec::SmallVec::<[_; MAX_CAP]>::new())
    )
);

bench_group!(
    bench_swap_remove_heap,
    "smallvec/swap_remove/heap",
    ("cds", bf_swap_remove!(HSV::with_capacity(MAX_CAP))),
    ("std", bf_swap_remove!(Vec::with_capacity(MAX_CAP))),
    ("tinyvec", bf_swap_remove!(HTV::with_capacity(MAX_CAP))),
    (
        "smallvec",
        bf_swap_remove!(smallvec::SmallVec::<[_; HEAP_CAP]>::with_capacity(MAX_CAP))
    )
);

bench_group!(
    bench_swap_remove_local,
    "smallvec/swap_remove/local",
    ("cds", bf_swap_remove!(cds::small_vec![MAX_CAP;])),
    ("std", bf_swap_remove!(Vec::new())),
    ("tinyvec", bf_swap_remove!(tinyvec::tiny_vec!([_; MAX_CAP]))),
    (
        "smallvec",
        bf_swap_remove!(smallvec::SmallVec::<[_; MAX_CAP]>::new())
    )
);

bench_group!(
    bench_resize_heap,
    "smallvec/resize/heap",
    ("cds", bf_resize!(HSV::with_capacity(MAX_CAP))),
    ("std", bf_resize!(Vec::with_capacity(MAX_CAP))),
    ("tinyvec", bf_resize!(HTV::with_capacity(MAX_CAP))),
    (
        "smallvec",
        bf_resize!(smallvec::SmallVec::<[_; HEAP_CAP]>::with_capacity(MAX_CAP))
    )
);

bench_group!(
    bench_resize_local,
    "smallvec/resize/local",
    ("cds", bf_resize!(cds::small_vec![MAX_CAP;])),
    ("std", bf_resize!(Vec::new())),
    ("tinyvec", bf_resize!(tinyvec::tiny_vec!([_; MAX_CAP]))),
    (
        "smallvec",
        bf_resize!(smallvec::SmallVec::<[_; MAX_CAP]>::new())
    )
);

bench_group!(
    bench_drain_heap,
    "smallvec/drain/heap",
    ("cds", bf_drain!(HSV::with_capacity(MAX_CAP))),
    ("std", bf_drain!(Vec::with_capacity(MAX_CAP))),
    ("tinyvec", bf_drain!(HTV::with_capacity(MAX_CAP))),
    (
        "smallvec",
        bf_drain!(smallvec::SmallVec::<[_; HEAP_CAP]>::with_capacity(MAX_CAP))
    )
);

bench_group!(
    bench_drain_local,
    "smallvec/drain/local",
    ("cds", bf_drain!(cds::small_vec![MAX_CAP;])),
    ("std", bf_drain!(Vec::new())),
    ("tinyvec", bf_drain!(tinyvec::tiny_vec!([_; MAX_CAP]))),
    (
        "smallvec",
        bf_drain!(smallvec::SmallVec::<[_; MAX_CAP]>::new())
    )
);

bench_group!(
    bench_drain_no_consume_heap,
    "smallvec/drain_no_consume/heap",
    ("cds", bf_drain_no_consume!(HSV::with_capacity(MAX_CAP))),
    ("std", bf_drain_no_consume!(Vec::with_capacity(MAX_CAP))),
    ("tinyvec", bf_drain_no_consume!(HTV::with_capacity(MAX_CAP))),
    (
        "smallvec",
        bf_drain_no_consume!(smallvec::SmallVec::<[_; HEAP_CAP]>::with_capacity(MAX_CAP))
    )
);

bench_group!(
    bench_drain_no_consume_local,
    "smallvec/drain_no_consume/local",
    ("cds", bf_drain_no_consume!(cds::small_vec![MAX_CAP;])),
    ("std", bf_drain_no_consume!(Vec::new())),
    (
        "tinyvec",
        bf_drain_no_consume!(tinyvec::tiny_vec!([_; MAX_CAP]))
    ),
    (
        "smallvec",
        bf_drain_no_consume!(smallvec::SmallVec::<[_; MAX_CAP]>::new())
    )
);

criterion_group!(
    benches,
    bench_push_heap,
    bench_push_local,
    bench_pop_heap,
    bench_pop_local,
    bench_insert_heap,
    bench_insert_local,
    bench_remove_heap,
    bench_remove_local,
    bench_swap_remove_heap,
    bench_swap_remove_local,
    bench_resize_heap,
    bench_resize_local,
    bench_drain_heap,
    bench_drain_local,
    bench_drain_no_consume_heap,
    bench_drain_no_consume_local
);
criterion_main!(benches);
