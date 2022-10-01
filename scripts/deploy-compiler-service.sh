#!/bin/bash -eux
cd $(realpath $(dirname $0))/..
PROJECT=us-west1-docker.pkg.dev/oort-319301
CONTAINER_IMAGE=$PROJECT/services/oort_compiler_service
docker tag oort_compiler_service:latest $CONTAINER_IMAGE
docker push $CONTAINER_IMAGE
gcloud run deploy oort-compiler-service \
  --image $CONTAINER_IMAGE \
  --allow-unauthenticated \
  --region=us-west1 \
  --cpu 1 \
  --memory 1G \
  --timeout 20s \
  --concurrency 1 \
  --max-instances 3 \
  --service-account=oort-compiler-service@oort-319301.iam.gserviceaccount.com
gcloud compute ssh server-1 --command="docker image prune --force" || true
gcloud compute instances update-container \
  server-1 \
  --container-image $CONTAINER_IMAGE
