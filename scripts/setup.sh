#!/bin/bash -eu
eval "$(fnm env)"

cd ../backend
fnm use
npm install
