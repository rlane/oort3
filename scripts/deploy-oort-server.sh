#!/bin/bash -eux
PROJECT=us-west1-docker.pkg.dev/oort-319301
gcloud run deploy oortserver \
  --image $PROJECT/oortserver/oortserver \
  --allow-unauthenticated \
  --region=us-west1 \
  --memory 2G
