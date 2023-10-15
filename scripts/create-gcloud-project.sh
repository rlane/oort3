#!/bin/bash -eux
# This script creates a new Google Cloud project running Oort.
# Prerequisites:
#   - gcloud CLI installed and logged in
#   - A Google Cloud billing account
#   - Docker installed and logged in to gcloud
#   - cargo install fnm trunk wasm-opt
which gcloud
which fnm
which trunk

PROJECT_ID=${1:-"oort-staging-$(date +'%Y%m%d-%H%M%S')"}
REGION=us-west1
BILLING_ACCOUNT=$(gcloud beta billing accounts list --filter=open=true --limit=1 --format='value(name)')
test -n "$BILLING_ACCOUNT"

export CLOUDSDK_CORE_PROJECT=$PROJECT_ID
gcloud projects create $PROJECT_ID
gcloud beta billing projects link $PROJECT_ID --billing-account $BILLING_ACCOUNT
gcloud services enable artifactregistry.googleapis.com run.googleapis.com firestore.googleapis.com appengine.googleapis.com firebase.googleapis.com cloudscheduler.googleapis.com

gcloud iam service-accounts create oort-backend-service
gcloud iam service-accounts create oort-compiler-service

for ROLE in roles/datastore.user roles/run.invoker; do
  gcloud projects add-iam-policy-binding $PROJECT_ID --member="serviceAccount:oort-backend-service@${PROJECT_ID}.iam.gserviceaccount.com" --role=$ROLE
done

while ! gcloud --project $PROJECT_ID artifacts repositories create services --repository-format=docker --location=$REGION; do
  sleep 10
done

gcloud --project $PROJECT_ID app create --region=$REGION
while ! gcloud --project $PROJECT_ID firestore databases update --type=firestore-native; do
  sleep 10
done

(cd firebase && eval "$(fnm env)" && fnm use && npx firebase --project $PROJECT_ID projects:addfirebase $PROJECT_ID)

# Push services first to get URLs
cargo oort-release -s --skip-git-checks --skip-github --no-secrets --project $PROJECT_ID -c backend,compiler
cargo oort-release -s --skip-git-checks --skip-github --no-secrets --project $PROJECT_ID -c app,tools
