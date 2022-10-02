#!/bin/bash -eux
cd $(realpath $(dirname $0))/..
rm -rf scratch/compiler_service_sandbox
mkdir -p scratch/compiler_service_sandbox
cp Cargo.toml.user scratch/compiler_service_sandbox/Cargo.toml
cp Cargo.lock.user scratch/compiler_service_sandbox/Cargo.lock
cp -a ai api scratch/compiler_service_sandbox
mkdir scratch/compiler_service_sandbox/scripts
cp scripts/build-ai.sh scripts/build-ai-fast.sh scratch/compiler_service_sandbox/scripts/
cargo build -p oort_compiler_service
cd scratch/compiler_service_sandbox
./scripts/build-ai.sh
./scripts/build-ai-fast.sh
PORT=8081 RUST_LOG=debug ../../target/debug/oort_compiler_service
