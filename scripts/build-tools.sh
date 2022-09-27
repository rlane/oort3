#!/bin/bash -eux
cd $(realpath $(dirname $0))/..
cargo build -p oort_tools --target-dir target.tools
mkdir -p scratch/tools
cp target.tools/debug/{tournament,battle,rescore,telemetry} scratch/tools/
