#!/bin/sh

cargo build -p flo_binding --release --target wasm32-unknown-unknown
cargo build -p flo_curves --release --target wasm32-unknown-unknown
cargo build -p flo_ui --release --target wasm32-unknown-unknown
cargo build -p flo_animation --release --target wasm32-unknown-unknown
cargo build -p flo_http_ui --release --target wasm32-unknown-unknown --features http
