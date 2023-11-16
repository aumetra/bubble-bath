use criterion::{black_box, criterion_group, criterion_main, Criterion};

const AMMONIA_INTRO: &str = r#"
<p>Ammonia is a whitelist-based HTML sanitization library. It is designed to
take untrusted user input with some HTML.</p>
<p>Because Ammonia uses <a href="https://github.com/servo/html5ever" title="The HTML parser in Servo">html5ever</a> to parse document fragments the same way
browsers do, it is extremely resilient to unknown attacks, much more so
than regular-expression-based sanitizers.</p>
<p>This library&#39;s API is modeled after <a href="https://github.com/jsocol/bleach">jsocol&#39;s Bleach</a> library for Python,
but is not affiliated with it in any way. Unlike Bleach, it does not do
linkification, it only sanitizes URLs in existing links.</p>
"#;

fn ammonia_intro_bench(c: &mut Criterion) {
    c.bench_function("bubble_bath_ammonia_intro", |b| {
        b.iter(|| {
            let _ = bubble_bath::clean(black_box(AMMONIA_INTRO)).unwrap();
        })
    });

    c.bench_function("ammonia_ammonia_intro", |b| {
        b.iter(|| {
            let _ = ammonia::clean(black_box(AMMONIA_INTRO));
        })
    });
}

criterion_group!(ammonia_intro, ammonia_intro_bench);
criterion_main!(ammonia_intro);
