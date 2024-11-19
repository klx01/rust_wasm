

#cargo build --target wasm32-unknown-unknown
#wasm-bindgen --target no-modules --out-dir www/wasm --no-typescript target/wasm32-unknown-unknown/debug/rust_wasm.wasm

cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --target no-modules --out-dir www/wasm --no-typescript target/wasm32-unknown-unknown/release/rust_wasm.wasm


