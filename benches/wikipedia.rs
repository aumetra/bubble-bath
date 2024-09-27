use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn wikipedia_bench(c: &mut Criterion) {
    let wikipedia = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/benches/",
        "wikipedia.txt",
    ))
    .unwrap();

    c.bench_function("bubble_bath_wikipedia", |b| {
        b.iter(|| {
            let _ = bubble_bath::clean(black_box(&wikipedia)).unwrap();
        })
    });

    c.bench_function("ammonia_wikipedia", |b| {
        b.iter(|| {
            let _ = ammonia::clean(black_box(&wikipedia));
        })
    });
}

criterion_group!(wikipedia, wikipedia_bench);
criterion_main!(wikipedia);
