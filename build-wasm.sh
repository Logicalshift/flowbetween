#!/bin/sh

cd binding
cargo +nightly build --target wasm32-unknown-unknown
cd ..

cd ui
cargo +nightly build --target wasm32-unknown-unknown
cd ..

cd animation
cargo +nightly build --target wasm32-unknown-unknown
cd ..
