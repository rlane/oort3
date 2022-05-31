#!/bin/bash -eu
eval "$(fnm env)"
set -x

cd $(realpath $(dirname $0))/..
rm -rf target/wasm32-unknown-unknown/release
for PKG in oort_simulator oort_worker oort_renderer oort-app; do
  cargo build --target wasm32-unknown-unknown --release --package $PKG
done

cd app
rm -rf dist
trunk build --release

cd ../backend
fnm use
npx firebase deploy "$@"
