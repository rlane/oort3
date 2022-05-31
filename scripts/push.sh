#!/bin/bash -eu
eval "$(fnm env)"
set -x

cd $(realpath $(dirname $0))/../app
rm -rf dist/ ../target/wasm32-unknown-unknown/release/build/oort-* ../target/wasm32-unknown-unknown/release/*oort.*
trunk build --release

cd ../backend
fnm use
npx firebase deploy "$@"
