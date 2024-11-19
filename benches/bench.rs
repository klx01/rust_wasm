#![feature(test)]

extern crate test;
extern crate rust_wasm;

use std::convert::TryInto;

// run with 
// cargo +nightly bench

#[bench]
fn universe_ticks(b: &mut test::Bencher) {
    let non_zero_big = 400.try_into().unwrap();
    let mut field = rust_wasm::game_of_life::Field::new(non_zero_big, non_zero_big);
    // Field::generate_by_fn(non_zero_default, non_zero_default, |i| i % 2 == 0 || i % 7 == 0),

    b.iter(|| {
        field.update();
    });
}