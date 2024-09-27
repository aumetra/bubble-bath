#![allow(missing_docs)]

use divan::black_box;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

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

#[divan::bench]
fn bubble_bath() -> String {
    bubble_bath::clean(black_box(AMMONIA_INTRO)).unwrap()
}

#[divan::bench]
fn ammonia() -> String {
    ammonia::clean(black_box(AMMONIA_INTRO))
}

fn main() {
    divan::main();
}
