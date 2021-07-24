#!/bin/bash -eux
cargo license -ad > www/attribution.txt
cd www
echo >> attribution.txt
npx license-ls >> attribution.txt
