#!/bin/bash -eux
cd $(realpath $(dirname $0))/..
(cd services && cargo verify-project --frozen --locked)
DOCKER_BUILDKIT=1 docker build -f services/leaderboard/Dockerfile --tag oort_leaderboard_service .
