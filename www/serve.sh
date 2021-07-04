#!/bin/bash -eux
cd $(realpath $(dirname $0))/..
cargo build --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/debug/oort.wasm www/
which basic-http-server || cargo install basic-http-server
basic-http-server www
