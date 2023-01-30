#!/bin/bash -eu
time cargo run --manifest-path tools/Cargo.toml --bin release -- "$@"
