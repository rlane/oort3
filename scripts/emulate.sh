#!/bin/bash -eu
eval "$(fnm env)"
set -x

cd $(realpath $(dirname $0))/../www
fnm use
npx webpack build --mode=development --watch &
trap "kill $! || true" exit

cd ../backend
fnm use
npx firebase emulators:start "$@"
