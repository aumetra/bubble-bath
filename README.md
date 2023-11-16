# bubble-bath

Small and quick HTML sanitizer

## Usage

Add `bubble-bath` to your dependencies:

```notest
cargo add bubble-bath
```

Use the library:

```rust
let unsanitized = r#"<script>alert('XSS!')</script>"#;
let clean = bubble_bath::clean(unsanitized);
```

## License

`bubble-bath` is either licensed under the Apache-2.0 or MIT license, at your choosing.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you,
shall be licensed as above, without any additional terms or conditions.
