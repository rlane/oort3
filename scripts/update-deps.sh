#!/bin/bash -eux
cd $(realpath $(dirname $0))/..
for WORKSPACE in frontend services shared tools; do
  (cd $WORKSPACE && cargo update)
done


(cd frontend && cargo check -q --target wasm32-unknown-unknown)
(cd shared && cargo test -q && cargo check -q --target wasm32-unknown-unknown --no-default-features --features js)
(cd tools && cargo check -q)
(cd services && cargo check -q)
