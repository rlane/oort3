#!/bin/bash -eu
eval "$(fnm env)"

cd $(realpath $(dirname $0))/../yew
fnm use
npm install

cd ../backend
fnm use
npm install

cd functions
fnm use
npm install
