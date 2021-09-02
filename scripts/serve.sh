#!/bin/bash -eux
cd $(realpath $(dirname $0))/../yew
trunk serve
