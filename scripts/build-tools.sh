#!/bin/bash -eux
cargo build -p oort_tools
mkdir -p scratch/tools
cp target/debug/{tournament,battle,rescore,telemetry} scratch/tools/
