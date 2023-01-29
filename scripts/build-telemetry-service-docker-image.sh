#!/bin/bash -eux
cd $(realpath $(dirname $0))/..
(cd services && cargo verify-project --frozen --locked)
DOCKER_BUILDKIT=1 docker build -f services/telemetry/Dockerfile --tag oort_telemetry_service --build-arg DISCORD_TELEMETRY_WEBHOOK=${DISCORD_TELEMETRY_WEBHOOK} .
