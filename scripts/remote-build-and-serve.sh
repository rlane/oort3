#!/bin/bash -eu
cd $(realpath $(dirname $0))/..
cargo watch -s "./scripts/remote-build.sh $@ && simple-http-server --port=8080 --index --nocache app/dist"
