use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn adder(a: u64, b: u64) -> u64 {
    a + b
}

/*
example of running tests in browser
see cargo config for specifying test runner
setup:
do setup from lib.rs, and
brew install geckodriver
run:
cargo test --target wasm32-unknown-unknown
for debugging need to set NO_HEADLESS
NO_HEADLESS=1 cargo test --target wasm32-unknown-unknown
*/

#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod test {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);
    #[wasm_bindgen_test]
    fn test() {
        assert_eq!(4, adder(1, 3));
    }
}