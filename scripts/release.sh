#!/bin/bash -eux
cd $(realpath $(dirname $0))/..

PUSH_ALL=1
PUSH_APP=0
PUSH_COMPILER_SERVICE=0
PUSH_TELEMETRY_SERVICE=0
while getopts "ws" option; do
   case $option in
      w) PUSH_ALL=0; PUSH_APP=1;;
      c) PUSH_ALL=0; PUSH_COMPILER_SERVICE=1;;
      t) PUSH_ALL=0; PUSH_TELEMETRY_SERVICE=1;;
      \?) exit;;
   esac
done

cargo workspaces publish --all --no-individual-tags --force='*'

if [[ $PUSH_ALL -eq 1 || $PUSH_COMPILER_SERVICE  -eq 1 ]]; then
  scripts/build-compiler-service-docker-image.sh
  scripts/deploy-compiler-service.sh
fi

if [[ $PUSH_ALL -eq 1 || $PUSH_TELEMETRY_SERVICE  -eq 1 ]]; then
  scripts/build-telemetry-service-docker-image.sh
  scripts/deploy-telemetry-service.sh
fi

if [[ $PUSH_ALL -eq 1 || $PUSH_APP -eq 1 ]]; then
  scripts/push.sh
fi
