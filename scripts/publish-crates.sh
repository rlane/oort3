#!/bin/bash -eux
cargo workspaces version
cargo publish -p oort_shared
cargo publish -p oort_api
