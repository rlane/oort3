#!/bin/bash -eu
eval "$(fnm env)"
set -x

cd $(realpath $(dirname $0))/../yew
fnm use
npx webpack serve --mode=development "$@"
