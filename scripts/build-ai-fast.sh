#!/bin/sh -eu
exec rustc --crate-name oort_reference_ai \
  --edition=2021 \
  ai/src/lib.rs \
  --crate-type cdylib \
  -o output.wasm \
  --target wasm32-unknown-unknown \
  -C strip=debuginfo \
  -L dependency=target/wasm32-unknown-unknown/release/deps \
  --extern oorandom=$(echo target/wasm32-unknown-unknown/release/deps/liboorandom-*.rlib) \
  --extern oort_shared=$(echo target/wasm32-unknown-unknown/release/deps/liboort_shared-*.rlib) \
  --extern wee_alloc=$(echo target/wasm32-unknown-unknown/release/deps/libwee_alloc-*.rlib) \
  --extern heapless=$(echo target/wasm32-unknown-unknown/release/deps/libheapless-*.rlib) \
  -C opt-level=s \
  -C link-arg=-zstack-size=16384
