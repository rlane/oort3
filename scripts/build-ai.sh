#!/bin/bash -eux
export RUSTFLAGS="-C opt-level=s -C link-arg=-zstack-size=16384"
cargo build -v -j1 --frozen --locked --offline --release --target wasm32-unknown-unknown
