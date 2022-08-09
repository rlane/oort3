#!/bin/bash -eux
cd $(realpath $(dirname $0))/..
rm -rf scratch/server_sandbox
mkdir -p scratch/server_sandbox
cp Cargo.toml.user scratch/server_sandbox/Cargo.toml
cp Cargo.lock.user scratch/server_sandbox/Cargo.lock
cp -a ai api shared scratch/server_sandbox
mkdir scratch/server_sandbox/scripts
cp scripts/build-ai.sh scripts/build-ai-fast.sh scratch/server_sandbox/scripts/
cargo build -p oort_server
cd scratch/server_sandbox
./scripts/build-ai.sh
./scripts/build-ai-fast.sh
PORT=8081 RUST_LOG=debug ../../target/debug/oort_server
