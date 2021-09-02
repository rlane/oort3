#!/bin/bash -eu
eval "$(fnm env)"

which trunk || echo "Missing trunk, run 'cargo install trunk'"

cd ../backend
fnm use
npm install

cd functions
fnm use
npm install
