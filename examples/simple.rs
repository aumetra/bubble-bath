#![allow(missing_docs)]

use bubble_bath::BubbleBath;
use std::{
    error::Error,
    io::{self, Read},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let sanitizer = BubbleBath {
        preserve_escaped: true,
        ..BubbleBath::default()
    };
    let output = sanitizer.clean(&input).unwrap();
    println!("{output}");

    Ok(())
}
