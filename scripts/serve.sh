#!/bin/bash -eux
cd $(realpath $(dirname $0))/..
./scripts/wasm-bindgen-macroquad.sh oort
which basic-http-server || cargo install basic-http-server
basic-http-server www
