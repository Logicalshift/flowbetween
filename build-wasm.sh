#!/bin/sh

cd binding
cargo build --target wasm32-unknown-unknown
cd ..

cd curves
cargo build --target wasm32-unknown-unknown
cd ..

cd ui
cargo build --target wasm32-unknown-unknown
cd ..

cd animation
cargo build --target wasm32-unknown-unknown
cd ..
