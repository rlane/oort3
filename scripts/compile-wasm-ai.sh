#!/bin/bash -eux
export RUSTFLAGS="-C lto=on -C opt-level=s -C link-arg=-zstack-size=1024"
cargo build --target wasm32-unknown-unknown --release -p oort_reference_ai
WASM=target/wasm32-unknown-unknown/release/oort_reference_ai.wasm
wasm-opt -Oz -o $WASM $WASM
cp $WASM ai/reference.wasm
