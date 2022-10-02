#!/bin/bash -eu
eval "$(fnm env)"
set -x

cd $(realpath $(dirname $0))/../frontend/app
trunk serve --watch .. --watch ../../simulator --watch ../../api --watch ../../ai/builtin-ai.tar.gz "$@"
