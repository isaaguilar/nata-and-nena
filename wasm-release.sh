#!/bin/bash

# cleanup
rm -rf out
rm -rf dist

# optimized build
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./out --target web ./target/wasm32-unknown-unknown/release/nata-and-nena.wasm
trunk build --release --dist dist 

# release
# aws s3 sync dist/ "$S3_BUCKET"
