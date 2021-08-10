#!/bin/bash -eu
BUCKET=oort-bin
cd $(realpath $(dirname $0))/../www
if which fnm >/dev/null; then
  eval "$(fnm env)"
fi
set +x
rm -rf dist/ ../target/wasm32-unknown-unknown/release/build/oort-* ../target/wasm32-unknown-unknown/release/*oort.*
npx webpack build
npx firebase deploy "$@"
