#!/bin/bash -eux
cd $(realpath $(dirname $0))/..
PROJECT=us-west1-docker.pkg.dev/oort-319301
CONTAINER_IMAGE=$PROJECT/oortserver/oortserver
docker tag oort_server:latest $CONTAINER_IMAGE
docker push $CONTAINER_IMAGE
gcloud run deploy oortserver \
  --image $CONTAINER_IMAGE \
  --allow-unauthenticated \
  --region=us-west1 \
  --cpu 1 \
  --memory 1G \
  --timeout 20s \
  --concurrency 1 \
  --max-instances 3
gcloud compute instances update-container \
  server-1 \
  --container-image $CONTAINER_IMAGE
gcloud compute ssh server-1 --command="docker image prune --force"
