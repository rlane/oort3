#!/bin/bash -eux
BUCKET=oort-bin
cd $(realpath $(dirname $0))/../www
npm run build
gsutil -m rsync dist/. gs://$BUCKET/
: Pushed to https://storage.googleapis.com/$BUCKET/index.html
