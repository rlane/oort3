#!/bin/bash -eu
eval "$(fnm env)"
set -x

cd $(realpath $(dirname $0))/../www
fnm use
rm -rf dist/ ../target/wasm32-unknown-unknown/release/build/oort-* ../target/wasm32-unknown-unknown/release/*oort.*
npx webpack build --mode=production

cd ../backend
fnm use
npx firebase deploy "$@"
