#!/bin/bash -eu
cd $(realpath $(dirname $0))/..
eval "$(fnm env)"
set -x

cd frontend
cargo build --release --bins --target wasm32-unknown-unknown
cd app
rm -rf dist
trunk build --release

cd ../../firebase
fnm use
npx firebase deploy "$@"
