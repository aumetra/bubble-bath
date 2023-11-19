//! A subset of ammonia's tests adapted to `bubble-bath`

use bubble_bath::*;

#[test]
fn deeply_nested_allowlisted() {
    clean("<b>".repeat(60_000)).unwrap();
}

#[test]
fn deeply_nested_denylisted() {
    clean("<b-b>".repeat(60_000)).unwrap();
}

#[test]
fn deeply_nested_alternating() {
    clean("<b-b>".repeat(35_000)).unwrap();
}

#[test]
fn included_angles() {
    let fragment = "1 < 2";
    let result = clean(fragment).unwrap();
    assert_eq!(result, "1 &lt; 2");
}

#[test]
fn remove_script() {
    let fragment = "an <script>evil()</script> example";
    let result = clean(fragment).unwrap();
    assert_eq!(result, "an  example");
}

#[test]
fn ignore_link() {
    let fragment = "a <a href=\"http://www.google.com\">good</a> example";
    let expected = "a <a href=\"http://www.google.com\" rel=\"noopener noreferrer\">\
                        good</a> example";
    let result = clean(fragment).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn remove_unsafe_link() {
    let fragment = "an <a onclick=\"evil()\" href=\"http://www.google.com\">evil</a> example";
    let result = clean(fragment).unwrap();
    assert_eq!(
        result,
        "an <a href=\"http://www.google.com\" rel=\"noopener noreferrer\">evil</a> example"
    );
}

#[test]
fn remove_js_link() {
    let fragment = "an <a href=\"javascript:evil()\">evil</a> example";
    let result = clean(fragment).unwrap();
    assert_eq!(result, "an <a rel=\"noopener noreferrer\">evil</a> example");
}

#[test]
fn tag_rebalance() {
    let fragment = "<b>AWESOME!";
    let result = clean(fragment).unwrap();
    assert_eq!(result, "<b>AWESOME!</b>");
}
