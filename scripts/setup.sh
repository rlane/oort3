#!/bin/bash -eu
eval "$(fnm env)"

cd ../backend
fnm use
npm install

cd functions
fnm use
npm install
