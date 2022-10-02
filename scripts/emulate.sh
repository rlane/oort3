#!/bin/bash -eu
eval "$(fnm env)"
set -x

cd $(realpath $(dirname $0))/../frontend/app
trunk serve --watch .. "$@" &
trap "kill $! || true" exit

cd ../../firebase
fnm use
npx firebase emulators:start "$@"
