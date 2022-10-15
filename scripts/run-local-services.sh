#!/bin/bash -eu
(cd services && cargo build)
trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT
export RUST_LOG=debug
export ENVIRONMENT=dev
./scripts/run-local-compiler-service.sh &
(cd services && PORT=8082 exec cargo run -p oort_telemetry_service) &
(cd services && PORT=8083 exec cargo run -p oort_leaderboard_service) &
wait
