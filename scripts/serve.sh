#!/bin/bash -eux
cd $(realpath $(dirname $0))/../www
npm run serve
