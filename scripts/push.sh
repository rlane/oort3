#!/bin/bash -eu
cd $(realpath $(dirname $0))/..
eval "$(fnm env)"
set -x

cd app
rm -rf dist
if [ ! -z ${REMOTE_BUILD:-} ]; then
  ../scripts/remote-build.sh --release
else
  trunk build --release
fi

cd ../backend
fnm use
npx firebase deploy "$@"
