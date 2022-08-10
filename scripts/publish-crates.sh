#!/bin/bash -eux
cargo workspaces version --no-git-tag minor
cargo publish -p oort_shared
cargo publish -p oort_api
