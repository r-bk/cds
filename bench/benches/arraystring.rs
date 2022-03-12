use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::{Duration, Instant};

const CAP: usize = 65535 - 2;
type S = cds::arraystring::ArrayString<cds::len::U16, cds::mem::Uninitialized, CAP>;

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
    ($sctor: expr) => {
        |b| {
            b.iter_custom(|mut iters| {
                let mut total = Duration::new(0, 0);
                let mut s = $sctor;
                while iters > 0 {
                    let tmp = (CAP as u64).min(iters);
                    s.clear();
                    let start = Instant::now();
                    for _i in 0..tmp {
                        let _ = s.push('a');
                    }
                    total += start.elapsed();
                    iters -= tmp;
                }
                total
            });
        }
    };
}

macro_rules! bf_push_str {
    ($sctor: expr) => {
        |b| {
            b.iter_custom(|mut iters| {
                let mut total = Duration::new(0, 0);
                let mut s = $sctor;
                while iters > 0 {
                    let tmp = (CAP as u64).min(iters);
                    s.clear();
                    let start = Instant::now();
                    for _i in 0..tmp {
                        let _ = s.push_str("a");
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
    ($sctor: expr) => {
        |b| {
            b.iter_custom(|mut iters| {
                let mut total = Duration::new(0, 0);
                let mut s = $sctor;
                while iters > 0 {
                    let tmp = (CAP as u64).min(iters);
                    for _i in 0..tmp {
                        s.push('a');
                    }
                    let start = Instant::now();
                    for _i in 0..tmp {
                        assert_eq!(Some('a'), s.pop());
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
    ($sctor: expr) => {
        |b| {
            b.iter_custom(|mut iters| {
                let mut total = Duration::new(0, 0);
                let mut s = $sctor;
                while iters > 0 {
                    let tmp = (CAP as u64).min(iters);
                    s.clear();
                    let start = Instant::now();
                    for i in 0..tmp {
                        s.insert(i as usize, 'a');
                    }
                    total += start.elapsed();
                    iters -= tmp;
                    assert_eq!(s.len(), tmp as usize);
                }
                total
            });
        }
    };
}

macro_rules! bf_insert_str {
    ($sctor: expr) => {
        |b| {
            b.iter_custom(|mut iters| {
                let mut total = Duration::new(0, 0);
                let mut s = $sctor;
                while iters > 0 {
                    let tmp = (CAP as u64).min(iters);
                    s.clear();
                    let start = Instant::now();
                    for i in 0..tmp {
                        s.insert_str(i as usize, "a");
                    }
                    total += start.elapsed();
                    iters -= tmp;
                    assert_eq!(s.len(), tmp as usize);
                }
                total
            });
        }
    };
}

macro_rules! bf_remove {
    ($sctor: expr) => {
        |b| {
            b.iter_custom(|mut iters| {
                let mut total = Duration::new(0, 0);
                let mut s = $sctor;
                while iters > 0 {
                    let tmp = (CAP as u64).min(iters);
                    for _ in 0..tmp {
                        s.push('a');
                    }
                    let start = Instant::now();
                    for i in (0..tmp).rev() {
                        s.remove(i as usize);
                    }
                    total += start.elapsed();
                    iters -= tmp;
                    assert!(s.is_empty());
                }
                total
            });
        }
    };
}

macro_rules! bf_truncate {
    ($sctor: expr) => {
        |b| {
            b.iter_custom(|mut iters| {
                let mut total = Duration::new(0, 0);
                let mut s = $sctor;
                while iters > 0 {
                    let tmp = (CAP as u64).min(iters);
                    for _ in 0..tmp {
                        s.push('a');
                    }
                    let start = Instant::now();
                    for i in (0..tmp).rev() {
                        s.truncate(i as usize);
                    }
                    total += start.elapsed();
                    iters -= tmp;
                    assert!(s.is_empty());
                }
                total
            });
        }
    };
}

fn bench_lformat(c: &mut Criterion) {
    let mut group = c.benchmark_group("arraystring/lformat");
    group.bench_function("cds", |b| {
        b.iter(|| {
            black_box(cds::lformat!(8, "cds"));
        })
    });
    group.bench_function("std", |b| {
        b.iter(|| {
            black_box(std::format!("cds"));
        })
    });
    group.finish();
}

bench_group!(
    bench_push,
    "arraystring/push",
    ("cds", bf_push!(S::new())),
    ("std", bf_push!(String::with_capacity(CAP))),
    ("arrayvec", bf_push!(arrayvec::ArrayString::<CAP>::new()))
);

bench_group!(
    bench_push_str,
    "arraystring/push_str",
    ("cds", bf_push_str!(S::new())),
    ("std", bf_push_str!(String::with_capacity(CAP))),
    (
        "arrayvec",
        bf_push_str!(arrayvec::ArrayString::<CAP>::new())
    )
);

bench_group!(
    bench_pop,
    "arraystring/pop",
    ("cds", bf_pop!(S::new())),
    ("std", bf_pop!(String::with_capacity(CAP))),
    ("arrayvec", bf_pop!(arrayvec::ArrayString::<CAP>::new()))
);

bench_group!(
    bench_insert,
    "arraystring/insert",
    ("cds", bf_insert!(S::new())),
    ("std", bf_insert!(String::with_capacity(CAP)))
);

bench_group!(
    bench_insert_str,
    "arraystring/insert_str",
    ("cds", bf_insert_str!(S::new())),
    ("std", bf_insert_str!(String::with_capacity(CAP)))
);

bench_group!(
    bench_remove,
    "arraystring/remove",
    ("cds", bf_remove!(S::new())),
    ("std", bf_remove!(String::with_capacity(CAP))),
    ("arrayvec", bf_remove!(arrayvec::ArrayString::<CAP>::new()))
);

bench_group!(
    bench_truncate,
    "arraystring/truncate",
    ("cds", bf_truncate!(S::new())),
    ("std", bf_truncate!(String::with_capacity(CAP))),
    (
        "arrayvec",
        bf_truncate!(arrayvec::ArrayString::<CAP>::new())
    )
);

criterion_group!(
    benches,
    bench_push,
    bench_push_str,
    bench_pop,
    bench_insert,
    bench_insert_str,
    bench_remove,
    bench_truncate,
    bench_lformat,
);
criterion_main!(benches);
