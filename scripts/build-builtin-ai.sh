#!/bin/bash -eux
cd $(realpath $(dirname $0))/..
rm -rf scratch/builtin_ai_sandbox
mkdir -p scratch/builtin_ai_sandbox
cp Cargo.toml.user scratch/builtin_ai_sandbox/Cargo.toml
cp Cargo.lock.user scratch/builtin_ai_sandbox/Cargo.lock
cp -a ai api scratch/builtin_ai_sandbox
mkdir scratch/builtin_ai_sandbox/scripts
cp scripts/build-ai.sh scripts/build-ai-fast.sh scratch/builtin_ai_sandbox/scripts/
cd scratch/builtin_ai_sandbox
./scripts/build-ai.sh

cd ai

SRCS=$( (find -path ./src -prune -o -name '*.rs' -printf '%P\n') )
for SRC in $SRCS
do
  DST=${SRC/%.rs/.wasm}
  cp $SRC src/user.rs
  (cd .. && ./scripts/build-ai-fast.sh)
  wasm-opt -Oz -o $DST ../output.wasm
done

WASMS=$( (find -path ./src -prune -o -name '*.wasm' -printf '%P\n') )

tar -czf builtin-ai.tar.gz $SRCS $WASMS
cp builtin-ai.tar.gz ../../../ai/builtin-ai.tar.gz
