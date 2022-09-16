#!/bin/bash -eux
cd $(realpath $(dirname $0))/..
PROJECT_ID=oort-319301 cargo run -p oort_tools --bin rescore
