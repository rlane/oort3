#!/bin/bash -eu
eval "$(fnm env)"

cd ../firebase
fnm use
npm install
