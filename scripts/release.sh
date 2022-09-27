#!/bin/bash -eux
cd $(realpath $(dirname $0))/..

PUSH_ALL=1
PUSH_APP=0
PUSH_SERVER=0
while getopts "ws" option; do
   case $option in
      w) PUSH_ALL=0; PUSH_APP=1;;
      s) PUSH_ALL=0; PUSH_SERVER=1;;
      \?) exit;;
   esac
done

cargo workspaces publish --all --no-individual-tags --force='*'

if [[ $PUSH_ALL -eq 1 || $PUSH_SERVER  -eq 1 ]]; then
  scripts/build-docker-image.sh
  scripts/deploy-oort-server.sh
fi

if [[ $PUSH_ALL -eq 1 || $PUSH_APP -eq 1 ]]; then
  scripts/push.sh
fi
