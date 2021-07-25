#!/bin/bash -eux
cargo flamegraph --bench=tutorials --cmd="record --call-graph=lbr -g"
