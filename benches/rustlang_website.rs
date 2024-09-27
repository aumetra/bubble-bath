#![allow(missing_docs)]

use divan::black_box;
use std::fs;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn read_input() -> String {
    fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/benches/",
        "rustlang-website.txt",
    ))
    .unwrap()
}

#[divan::bench]
fn bubble_bath(bencher: divan::Bencher<'_, '_>) {
    let input = read_input();

    bencher.bench(|| bubble_bath::clean(black_box(&input)).unwrap());
}

#[divan::bench]
fn ammonia(bencher: divan::Bencher<'_, '_>) {
    let input = read_input();

    bencher.bench(|| ammonia::clean(black_box(&input)));
}

fn main() {
    divan::main();
}
