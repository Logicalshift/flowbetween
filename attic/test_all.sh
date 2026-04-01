#!/bin/sh

cd sqlite_storage
cargo test -p flo_sqlite_storage
cd ..

cargo test -p flo_float_encoder
cargo test -p flo_logging
cargo test -p flo_binding
cargo test -p flo_stream
cargo test -p flo_curves
cargo test -p flo_canvas
cargo test -p flo_ui
cargo test -p flo_ui_files
cargo test -p flo_animation
cargo test -p flo_static_files --features http
cargo test -p flo_http_ui --features http
cargo test -p flo_http_ui_actix --features http
cargo test -p flo
cargo test --features http
