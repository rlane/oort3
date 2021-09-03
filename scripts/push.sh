#!/bin/bash -eu
eval "$(fnm env)"
set -x

cd $(realpath $(dirname $0))/../yew
rm -rf dist/ ../target/wasm32-unknown-unknown/release/build/oort-* ../target/wasm32-unknown-unknown/release/*oort.*
trunk build --release --dist ../target/dist

cd ../backend
fnm use
npx firebase deploy "$@"
