#!/bin/bash -eu
cd $(realpath $(dirname $0))/../www
if which fnm >/dev/null; then
  eval "$(fnm env)"
fi
set +x
npx webpack build --mode=development
npx firebase emulators:start "$@"
