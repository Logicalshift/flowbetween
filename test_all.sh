#!/bin/sh

cargo test -p binding
cargo test -p curves
cargo test -p canvas
cargo test -p ui
cargo test -p animation
cargo test -p static_files
cargo test -p http_ui
cargo test -p flo
cargo test
