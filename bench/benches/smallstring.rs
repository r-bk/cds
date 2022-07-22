use cds::smallstring::SmallString;
use compact_str::CompactString;
use criterion::{criterion_group, criterion_main, Criterion};
use std::time::{Duration, Instant};

const MAX_CAP: usize = 23;

type SmartString = smartstring::SmartString<smartstring::LazyCompact>;

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

macro_rules! bf_push_str {
    ($sctor: expr) => {
        |b| {
            b.iter_custom(|mut iters| {
                let mut total = Duration::new(0, 0);
                let mut s = $sctor;
                while iters > 0 {
                    let tmp = (MAX_CAP as u64).min(iters);
                    s.clear();
                    let start = Instant::now();
                    for _i in 0..tmp {
                        s.push_str("a");
                    }
                    total += start.elapsed();
                    iters -= tmp;
                }
                total
            });
        }
    };
}

bench_group!(
    bench_push_str_heap,
    "smallstring/push/heap",
    (
        "cds",
        bf_push_str!(SmallString::<MAX_CAP>::with_capacity(1024))
    ),
    ("std", bf_push_str!(String::with_capacity(1024))),
    ("smartstring", bf_push_str!(SmartString::new())),
    (
        "compact_str",
        bf_push_str!(CompactString::with_capacity(1024))
    )
);

bench_group!(
    bench_push_str_local,
    "smallstring/push/local",
    (
        "cds",
        bf_push_str!(SmallString::<MAX_CAP>::with_capacity(MAX_CAP))
    ),
    ("std", bf_push_str!(String::with_capacity(MAX_CAP))),
    ("smartstring", bf_push_str!(SmartString::new())),
    (
        "compact_str",
        bf_push_str!(CompactString::with_capacity(MAX_CAP))
    )
);

criterion_group!(benches, bench_push_str_heap, bench_push_str_local);
criterion_main!(benches);
