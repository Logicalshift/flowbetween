#!/bin/sh

cargo test -p flo_float_encoder
cargo test -p flo_binding
cargo test -p flo_curves
cargo test -p flo_canvas
cargo test -p flo_ui
cargo test -p flo_animation
cargo test -p flo_anim_sqlite
cargo test -p flo_static_files
cargo test -p flo_http_ui
cargo test -p flo
cargo test
