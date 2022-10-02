#!/bin/bash -eux
cd $(realpath $(dirname $0))/../tools
cargo build
mkdir -p ../scratch/tools
cp target/debug/{tournament,battle,rescore,telemetry} ../scratch/tools/
