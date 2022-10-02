#!/bin/bash -eu
eval "$(fnm env)"
set -x

cd $(realpath $(dirname $0))/../frontend/app
trunk serve --watch .. --watch ../../shared/ai/builtin-ai.tar.gz --watch ../../shared/api  --watch ../../shared/simulator "$@"
