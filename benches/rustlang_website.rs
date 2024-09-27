use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;

fn rustlang_website_bench(c: &mut Criterion) {
    let rustlang_website = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/benches/",
        "rustlang-website.txt",
    ))
    .unwrap();

    c.bench_function("bubble_bath_rustlang_website", |b| {
        b.iter(|| {
            let _ = bubble_bath::clean(black_box(&rustlang_website)).unwrap();
        })
    });

    c.bench_function("ammonia_rustlang_website", |b| {
        b.iter(|| {
            let _ = ammonia::clean(black_box(&rustlang_website));
        })
    });
}

criterion_group!(rustlang_website, rustlang_website_bench);
criterion_main!(rustlang_website);
