#!/bin/bash -eux
cd $(realpath $(dirname $0))/..
cargo workspaces publish --all --no-individual-tags
scripts/build-docker-image.sh
scripts/deploy-oort-server.sh
scripts/push.sh
