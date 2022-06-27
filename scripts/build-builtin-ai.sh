#!/bin/bash -eux
cd $(realpath $(dirname $0))/..
rm -rf scratch/builtin_ai_sandbox
mkdir -p scratch/builtin_ai_sandbox
cp Cargo.toml.user scratch/builtin_ai_sandbox/Cargo.toml
cp Cargo.lock.user scratch/builtin_ai_sandbox/Cargo.lock
cp -a ai shared scratch/builtin_ai_sandbox
mkdir scratch/builtin_ai_sandbox/scripts
cp scripts/build-ai.sh scripts/build-ai-fast.sh scratch/builtin_ai_sandbox/scripts/
cd scratch/builtin_ai_sandbox
./scripts/build-ai.sh

AI_DIR=../../ai
mkdir -p $AI_DIR/compiled/tutorial

SRCS=$( (cd ai; find -path ./src -prune -o -name '*.rs' -not -name '*.initial.rs' -printf '%P\n') )
for SRC in $SRCS
do
  DST=${SRC/%.rs/.wasm}
  cp $AI_DIR/$SRC ai/src/user.rs
  ./scripts/build-ai-fast.sh
  mv output.wasm $AI_DIR/compiled/$DST
done
