#!/bin/bash -eu
eval "$(fnm env)"
set -x

cd $(realpath $(dirname $0))/../app
trunk serve --watch .. --ignore ../ai/src --ignore ../scratch "$@"
