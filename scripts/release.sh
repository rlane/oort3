#!/bin/bash -eux
cd $(realpath $(dirname $0))/..
scripts/publish-crates.sh
scripts/build-docker-image.sh
scripts/deploy-oort-server.sh
scripts/push.sh
