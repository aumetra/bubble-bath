use criterion::{black_box, criterion_group, criterion_main, Criterion};

const RUSTLANG_WEBSITE: &str = include_str!("rustlang-website.txt");

fn rustlang_website_bench(c: &mut Criterion) {
    c.bench_function("bubble_bath_rustlang_website", |b| {
        b.iter(|| {
            let _ = bubble_bath::clean(black_box(RUSTLANG_WEBSITE)).unwrap();
        })
    });

    c.bench_function("ammonia_rustlang_website", |b| {
        b.iter(|| {
            let _ = ammonia::clean(black_box(RUSTLANG_WEBSITE));
        })
    });
}

criterion_group!(rustlang_website, rustlang_website_bench);
criterion_main!(rustlang_website);
