#!/bin/bash -eu
eval "$(fnm env)"

cd $(realpath $(dirname $0))/../www
fnm use
npm install

cd ../backend
fnm use
npm install
