use bubble_bath::BubbleBath;
use insta::assert_snapshot;
use std::fs;

#[test]
fn torture() {
    insta::glob!("inputs/*", |path| {
        let input = fs::read_to_string(path).unwrap();
        assert_snapshot!(bubble_bath::clean(&input).unwrap());
    });
}

#[test]
fn torture_escaped() {
    insta::glob!("inputs/*", |path| {
        let input = fs::read_to_string(path).unwrap();
        let bubble_bath = BubbleBath {
            preserve_escaped: true,
            ..BubbleBath::default()
        };

        assert_snapshot!(bubble_bath.clean(&input).unwrap());
    });
}
