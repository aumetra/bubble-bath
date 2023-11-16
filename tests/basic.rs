use bubble_bath::BubbleBath;
use insta::assert_snapshot;

#[test]
fn script() {
    assert_snapshot!(bubble_bath::clean(
        r#"
            <script>alert('XSS!');</script>
            <p>Hello world!</p>
            <font size="20">LARGE</font>
        "#
    )
    .unwrap());
}

#[test]
fn preserve_escaped() {
    let bubble_bath = BubbleBath {
        preserve_escaped: true,
        ..BubbleBath::default()
    };

    assert_snapshot!(bubble_bath
        .clean(
            r#"
            <p>Hello world!</p>
            <font size="20">LARGE</font>
        "#
        )
        .unwrap());
}
