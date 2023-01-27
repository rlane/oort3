#!/bin/bash -eux
cd $(realpath $(dirname $0))/..
(cd services && cargo verify-project --frozen --locked)
DOCKER_BUILDKIT=1 docker build -f services/leaderboard/Dockerfile --tag oort_leaderboard_service --build-arg OORT_ENVELOPE_SECRET=${OORT_ENVELOPE_SECRET}  --build-arg OORT_CODE_ENCRYPTION_SECRET=${OORT_CODE_ENCRYPTION_SECRET} .
