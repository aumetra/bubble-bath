use criterion::{black_box, criterion_group, criterion_main, Criterion};

const WIKIPEDIA: &str = include_str!("wikipedia.txt");

fn wikipedia_bench(c: &mut Criterion) {
    c.bench_function("bubble_bath_wikipedia", |b| {
        b.iter(|| {
            let _ = bubble_bath::clean(black_box(WIKIPEDIA)).unwrap();
        })
    });

    c.bench_function("ammonia_wikipedia", |b| {
        b.iter(|| {
            let _ = ammonia::clean(black_box(WIKIPEDIA));
        })
    });
}

criterion_group!(wikipedia, wikipedia_bench);
criterion_main!(wikipedia);
