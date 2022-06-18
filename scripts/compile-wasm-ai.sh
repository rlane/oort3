#!/bin/bash -eux
rustc --crate-type=cdylib --target wasm32-unknown-unknown ai/rust/lib.rs -o ai/reference.wasm -C lto=on -C opt-level=s -C link-arg=-zstack-size=1024
wasm-opt -Oz -o ai/reference.wasm ai/reference.wasm
