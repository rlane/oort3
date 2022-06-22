#!/bin/bash -eux
export RUSTFLAGS="-C lto=on -C opt-level=s -C link-arg=-zstack-size=1024"
cargo build --frozen --locked --offline --release --target wasm32-unknown-unknown --message-format=json-diagnostic-short
