#!/bin/bash -eux
cd $(realpath $(dirname $0))/..
PROJECT=us-west1-docker.pkg.dev/oort-319301
docker tag oort_server:latest $PROJECT/oortserver/oortserver
docker push $PROJECT/oortserver/oortserver
gcloud run deploy oortserver \
  --image $PROJECT/oortserver/oortserver \
  --allow-unauthenticated \
  --region=us-west1 \
  --cpu 1 \
  --memory 1G \
  --timeout 20s
