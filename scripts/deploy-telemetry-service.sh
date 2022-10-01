#!/bin/bash -eux
cd $(realpath $(dirname $0))/..
PROJECT=us-west1-docker.pkg.dev/oort-319301
CONTAINER_IMAGE=$PROJECT/services/oort_telemetry_service
docker tag oort_telemetry_service:latest $CONTAINER_IMAGE
docker push $CONTAINER_IMAGE
gcloud run deploy oort-telemetry-service \
  --image $CONTAINER_IMAGE \
  --allow-unauthenticated \
  --region=us-west1 \
  --cpu 1 \
  --memory 1G \
  --timeout 20s \
  --concurrency 1 \
  --max-instances 3 \
  --service-account=oort-telemetry-service@oort-319301.iam.gserviceaccount.com
