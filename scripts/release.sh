#!/bin/bash -eu
cd $(realpath $(dirname $0))/..

function usage() {
  echo "Usage: $0 -[wctldsnh]"
  echo
  echo "  -w: Push app"
  echo "  -c: Push compiler service"
  echo "  -t: Push telemetry service"
  echo "  -l: Push leaderboard service"
  echo "  -d: Push docs"
  echo "  -s: Skip bumping version"
  echo "  -n: Dry run, skip deploying"
  echo "  -h: Display this message"
}

BUMP_VERSION=1
PUSH_ALL=1
PUSH_APP=0
PUSH_COMPILER_SERVICE=0
PUSH_TELEMETRY_SERVICE=0
PUSH_LEADERBOARD_SERVICE=0
PUSH_DOC=0
DRY_RUN=0
while getopts "wctldsnh" option; do
   case $option in
      w) PUSH_ALL=0; PUSH_APP=1;;
      c) PUSH_ALL=0; PUSH_COMPILER_SERVICE=1;;
      t) PUSH_ALL=0; PUSH_TELEMETRY_SERVICE=1;;
      l) PUSH_ALL=0; PUSH_LEADERBOARD_SERVICE=1;;
      d) PUSH_ALL=0; PUSH_DOC=1;;
      s) BUMP_VERSION=0;;
      n) DRY_RUN=1;;
      h) usage; exit;;
      \?) usage; exit;;
   esac
done

if ! git diff HEAD --quiet; then
  echo "Uncommitted changes, halting release"
  git diff HEAD
  exit 1
fi

[ -e scratch/secrets.sh ] && source scratch/secrets.sh

if [[ $BUMP_VERSION -eq 1 ]]; then
  (cd frontend && cargo workspaces version --all --force='*' --no-git-commit --yes)
  VERSION=$(egrep '^version = ".*"$' frontend/app/Cargo.toml | head -n1 | egrep -o '[0-9][^"]*')
  sed -i  "1s;^;### ${VERSION}\n\n;" CHANGELOG.md
  for WS in tools shared services; do
    (cd $WS && cargo workspaces version --all --force='*' --no-git-commit --yes custom $VERSION)
  done
  for WS in frontend tools shared services; do
    (cd $WS && cargo update -w)
  done
  git commit -n -a -m "bump version to $VERSION"
  git tag v$VERSION
fi

if [[ $PUSH_ALL -eq 1 || $PUSH_APP -eq 1 ]]; then
  echo "Building frontend"
  (
    set -x
    cd frontend
    cargo build --release --bins --target wasm32-unknown-unknown
    cd app
    rm -rf dist
    trunk build --release
  )
fi

if [[ $PUSH_ALL -eq 1 || $PUSH_COMPILER_SERVICE  -eq 1 ]]; then
  scripts/build-compiler-service-docker-image.sh
fi

if [[ $PUSH_ALL -eq 1 || $PUSH_TELEMETRY_SERVICE  -eq 1 ]]; then
  scripts/build-telemetry-service-docker-image.sh
fi

if [[ $PUSH_ALL -eq 1 || $PUSH_LEADERBOARD_SERVICE  -eq 1 ]]; then
  scripts/build-leaderboard-service-docker-image.sh
fi

if [[ $PUSH_ALL -eq 1 || $PUSH_DOC -eq 1 ]]; then
  (cd shared && cargo doc -p oort_api)
fi

if [[ $DRY_RUN -eq 1 ]]; then
  echo "Dry run, exiting"
  exit 0
fi

if [[ $PUSH_ALL -eq 1 || $PUSH_APP -eq 1 ]]; then
  echo "Deploying frontend"
  (
    cd firebase
    eval "$(fnm env)"
    set -x
    fnm use
    npx firebase deploy
  )
fi

if [[ $PUSH_ALL -eq 1 || $PUSH_COMPILER_SERVICE  -eq 1 ]]; then
  scripts/deploy-compiler-service.sh
fi

if [[ $PUSH_ALL -eq 1 || $PUSH_TELEMETRY_SERVICE  -eq 1 ]]; then
  scripts/deploy-telemetry-service.sh
fi

if [[ $PUSH_ALL -eq 1 || $PUSH_LEADERBOARD_SERVICE  -eq 1 ]]; then
  scripts/deploy-leaderboard-service.sh
fi

if [[ $PUSH_ALL -eq 1 || $PUSH_DOC -eq 1 ]]; then
  (cd shared && cargo publish -p oort_api)
fi
