#!/bin/bash -eux
# Inspired by https://github.com/sgeisler/cargo-remote.
# To set up remote building:
# 1. Install rust with rustup.
# 2. rustup target add wasm32-unknown-unknown
# 3. cargo install trunk
cd $(realpath $(dirname $0))/..
BUILDER=builder
REMOTE_DIR=oort3
rsync -a --delete --compress --checksum --info=progress2 \
  --exclude=target --exclude=dist --exclude=pkg --exclude=wbindgen \
  --exclude=backend --exclude=scratch --exclude=".*" \
  --rsync-path "mkdir -p remote-builds && rsync" \
  . $BUILDER:remote-builds/$REMOTE_DIR
ssh $BUILDER time trunk build remote-builds/$REMOTE_DIR/app/index.html "$@"
rsync -a --delete --compress --info=progress2 \
  $BUILDER:remote-builds/$REMOTE_DIR/app/dist app/
# To serve: simple-http-server --port=8080 app/dist
