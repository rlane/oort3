#!/bin/bash -eux
cd $(realpath $(dirname $0))/..
cargo verify-project --frozen --locked
DOCKER_BUILDKIT=1 docker build -f server/Dockerfile --tag oort_server .
