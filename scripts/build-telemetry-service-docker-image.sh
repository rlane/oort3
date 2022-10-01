#!/bin/bash -eux
cd $(realpath $(dirname $0))/..
cargo verify-project --frozen --locked
DOCKER_BUILDKIT=1 docker build -f services/telemetry/Dockerfile --tag oort_telemetry_service .
