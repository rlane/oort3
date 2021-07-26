#!/bin/bash -eu
BUCKET=oort-bin
cd $(realpath $(dirname $0))/../www
if which fnm >/dev/null; then
  eval "$(fnm env)"
fi
set +x
rm -rf ../target/wasm32-unknown-unknown/release/build/oort-* ../target/wasm32-unknown-unknown/release/*oort.*
npm run build
gsutil -m rsync dist/. gs://$BUCKET/
: Pushed to https://storage.googleapis.com/$BUCKET/index.html
