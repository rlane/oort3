#!/bin/bash -eu
eval "$(fnm env)"
set -x

cd $(realpath $(dirname $0))/../yew
trunk serve --watch .. "$@" &
trap "kill $! || true" exit

cd ../backend
fnm use
npx firebase emulators:start "$@"
