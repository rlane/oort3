#!/bin/bash -eu
cargo build --manifest-path services/Cargo.toml
trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT
export RUST_LOG=none,oort_leaderboard_service=debug,oort_compiler_service=debug,oort_telemetry_service=debug,oort_shortcode_service=debug
export ENVIRONMENT=dev
rm -rf /tmp/oort-ai
cargo run -q --manifest-path services/Cargo.toml -p oort_compiler_service -- --prepare
PORT=8081 cargo run -q --manifest-path services/Cargo.toml -p oort_compiler_service &
PORT=8082 cargo run -q --manifest-path services/Cargo.toml -p oort_telemetry_service &
PORT=8083 cargo run -q --manifest-path services/Cargo.toml -p oort_leaderboard_service &
PORT=8084 cargo run -q --manifest-path services/Cargo.toml -p oort_shortcode_service &
wait
